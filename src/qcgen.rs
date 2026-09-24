mod card;
mod cli;
mod data;
mod source;
mod subdivision;

use std::{collections::HashMap, env, fs::read_to_string, io::stdout, path::Path, sync::LazyLock};

use adif_reader::document::Record as AdifRecord;
use anyhow::{Context, Result, bail};
use callfind::grid_locator::GridLocator;
use compact_str::{CompactString, ToCompactString};
use mlua::prelude::*;
use regex::Regex;
use secrecy::SecretString;
use time::UtcOffset;
use tracing::{Level, info, span, warn};

use crate::{
    qcgen::{
        card::{QslCard, QslCardEntry, QslInfo, QslInstrument, QslOperator, QslStation},
        data::{Instrument, Operator, read_items_from_tomls},
        source::AdifSource,
        subdivision::SubdivisionResolver,
    },
    qso::{exchange::QsoExchanges, qsl::QslStatus, record::QsoRecord},
    schope::engine::{initialize_lua, lua_to_json},
    wavelog::{QsoQuery, WavelogClient},
};

pub use cli::Arguments;

const TOKEN_ENV: &str = "WAVELOG_TOKEN";

static RE_EXTRA_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!(\w+):([^\s]+)"#).expect("valid regex"));

struct EntryContext<'a> {
    instruments: HashMap<String, Instrument>,
    operators: HashMap<String, Operator>,
    default_instrument: Option<&'a str>,
    default_power: Option<f64>,
    subdivisions: Option<SubdivisionResolver<'a>>,
}

pub fn run(args: Arguments) -> Result<()> {
    let script_path = args
        .script_path
        .canonicalize()
        .with_context(|| format!("failed to open {}", args.script_path.display()))?;
    let script_args: HashMap<_, _> = args
        .script_args
        .into_iter()
        .map(|a| (a.0.to_string(), a.1.unwrap_or_default().to_string()))
        .collect();

    let client = match &args.wavelog {
        Some(url) => {
            let token = read_token(args.wavelog_token_file.as_deref())?;
            Some(WavelogClient::new(url, token)?)
        }
        None => None,
    };
    let source = match (&client, args.adif) {
        (Some(client), _) => AdifSource::Wavelog(
            client,
            QsoQuery {
                station_ids: args.station_id,
                qso_since: args.qso_since,
                qso_until: args.qso_until,
            },
        ),
        (None, Some(path)) => AdifSource::File(path),
        (None, None) => bail!("no QSO source specified"),
    };
    let documents = source.read_documents(args.lenient_length.unwrap_or_default().into())?;

    let mut context = EntryContext {
        instruments: read_items_from_tomls(args.instruments_files)?,
        operators: read_items_from_tomls(args.operators_files)?,
        default_instrument: args.instrument.as_deref(),
        default_power: args.power,
        subdivisions: client
            .as_ref()
            .filter(|_| !args.no_state_names)
            .map(SubdivisionResolver::new),
    };

    let mut entries = vec![];
    let records = documents.iter().flat_map(|d| d.records());
    for (i, record) in records.enumerate() {
        let span = span!(Level::ERROR, "record_process", index = i);
        let _enter = span.enter();

        let qsl_status = QslStatus::new(record)?;
        if !args.all_qsos && !qsl_status.should_send() {
            continue;
        }
        entries.push(build_entry(record, qsl_status, &mut context)?);
    }
    info!("processing {} QSOs", entries.len());

    let processed_json = run_script(&script_path, script_args, entries)?;
    serde_json::to_writer(stdout().lock(), &processed_json)?;

    Ok(())
}

fn read_token(token_file: Option<&Path>) -> Result<SecretString> {
    let token = match token_file {
        Some(path) => {
            read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?
        }
        None => env::var(TOKEN_ENV).with_context(|| {
            format!("Wavelog token not found; specify --wavelog-token-file or set {TOKEN_ENV}")
        })?,
    };
    Ok(SecretString::from(token.trim()))
}

fn build_entry(
    record: &AdifRecord,
    qsl_status: QslStatus,
    context: &mut EntryContext,
) -> Result<QslCardEntry> {
    let qso_record = QsoRecord::new(record, UtcOffset::UTC)?;
    let qso_exchanges = QsoExchanges::new(record);

    let mut instrument_key = context.default_instrument;
    let comment = field(record, "COMMENT").unwrap_or_default();
    for extra_tag in RE_EXTRA_TAG.captures_iter(comment) {
        let key = extra_tag.get(1).expect("group must exist");
        let value = extra_tag.get(2).expect("group must exist");
        match key.as_str() {
            "inst" => instrument_key = Some(value.as_str()),
            otherwise => warn!("unknown extra tag: {otherwise}"),
        }
    }

    let instrument = instrument_key.and_then(|k| context.instruments.get(k));
    if let (Some(key), None) = (instrument_key, instrument) {
        warn!("unknown instrument: {key}");
    }
    let power = field(record, "TX_PWR")
        .and_then(|p| p.parse().ok())
        .or(context.default_power)
        .or(instrument.and_then(|i| i.default_power));

    let operator_callsign = field(record, "OPERATOR");
    let operator_name = operator_callsign
        .and_then(|c| context.operators.get(c))
        .map(|o| o.name.to_compact_string());

    Ok(QslCardEntry {
        qso: qso_record.into(),
        exchange: qso_exchanges.into(),
        info: QslInfo {
            station: build_station(record, context.subdivisions.as_mut()),
            operator: QslOperator {
                callsign: operator_callsign.map(|s| s.to_compact_string()),
                name: operator_name,
            },
            instrument: QslInstrument {
                antenna: instrument.map(|i| i.antenna.to_compact_string()),
                rig: instrument.map(|i| i.rig.to_compact_string()),
                power,
            },
            card: QslCard {
                should_send: qsl_status.should_send(),
                received: qsl_status.received(),
                manager: compact_field(record, "QSL_VIA"),
            },
        },
    })
}

fn build_station(
    record: &AdifRecord,
    subdivisions: Option<&mut SubdivisionResolver>,
) -> QslStation {
    let grid = field(record, "MY_GRIDSQUARE").and_then(|g| match g.parse::<GridLocator>() {
        Ok(grid) => Some(grid),
        Err(e) => {
            warn!("invalid MY_GRIDSQUARE {g}: {e}");
            None
        }
    });

    let state = compact_field(record, "MY_STATE");
    let dxcc = field(record, "MY_DXCC").and_then(|d| d.parse::<u32>().ok());
    let state_name = match (subdivisions, dxcc, &state) {
        (Some(resolver), Some(dxcc), Some(state)) => resolver.resolve(dxcc, state),
        _ => None,
    };

    QslStation {
        callsign: compact_field(record, "STATION_CALLSIGN"),
        grid,
        city: compact_field(record, "MY_CITY"),
        state,
        state_name,
        country: compact_field(record, "MY_COUNTRY"),
        pota: compact_field(record, "MY_POTA_REF"),
        sota: compact_field(record, "MY_SOTA_REF"),
        wwff: compact_field(record, "MY_WWFF_REF"),
        iota: compact_field(record, "MY_IOTA"),
        sig: compact_field(record, "MY_SIG"),
        sig_info: compact_field(record, "MY_SIG_INFO"),
    }
}

fn field<'a>(record: &'a AdifRecord, name: &str) -> Option<&'a str> {
    record.field(name).filter(|v| !v.is_empty())
}

fn compact_field(record: &AdifRecord, name: &str) -> Option<CompactString> {
    field(record, name).map(|v| v.to_compact_string())
}

fn run_script(
    script_path: &Path,
    script_args: HashMap<String, String>,
    entries: Vec<QslCardEntry>,
) -> Result<serde_json::Value> {
    let script_text = read_to_string(script_path)
        .with_context(|| format!("failed to read {}", script_path.display()))?;
    let Some(script_base) = script_path.parent() else {
        bail!("script path is invalid");
    };

    let lua = initialize_lua(script_base)?;
    let script_table: LuaTable = lua.load(script_text).eval()?;
    let generate: LuaFunction = script_table.get("generate")?;
    let processed_value: LuaValue = generate.call((script_args, entries))?;

    Ok(lua_to_json(processed_value)?)
}
