mod card;
mod cli;
mod data;
mod source;
mod subdivision;

use std::{
    collections::HashMap,
    fs::read_to_string,
    io::stdout,
    path::{Path, absolute},
    sync::LazyLock,
};

use adif_reader::document::Record as AdifRecord;
use anyhow::{Context, Result, bail};
use callfind::grid_locator::GridLocator;
use compact_str::{CompactString, ToCompactString};
use mlua::prelude::*;
use regex::Regex;
use time::UtcOffset;
use tracing::{Level, info, span, warn};

use crate::{
    config::{Config, OperatorConfig},
    qcgen::{
        card::{QslCard, QslCardEntry, QslInfo, QslInstrument, QslOperator, QslStation},
        data::{Instrument, read_items_from_tomls},
        source::AdifSource,
        subdivision::SubdivisionResolver,
    },
    qso::{exchange::QsoExchanges, get_optional_field, qsl::QslStatus, record::QsoRecord},
    schope::engine::{initialize_lua, lua_to_json},
    wavelog::QsoQuery,
};

pub use cli::Arguments;

const INSTRUMENTS_FILENAME: &str = "instruments.toml";

static RE_EXTRA_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!(\w+):(\S+)").expect("valid regex"));

struct EntryContext<'a> {
    instruments: HashMap<String, Instrument>,
    operators: &'a HashMap<String, OperatorConfig>,
    default_instrument: Option<&'a str>,
    default_power: Option<f64>,
    subdivisions: Option<SubdivisionResolver<'a>>,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let script_path = absolute(&args.script_path)
        .with_context(|| format!("invalid script path {}", args.script_path.display()))?;
    let script_args: HashMap<_, _> = args
        .script_args
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    let client = config.wavelog_client()?;
    let source = match (args.adif, &client) {
        (Some(path), _) => AdifSource::File(path),
        (None, Some(client)) => AdifSource::Wavelog(client, QsoQuery::from(args.query)),
        (None, None) => bail!(
            "Wavelog is not configured; add [wavelog] to {} or specify --adif",
            config.path().display()
        ),
    };
    let documents = source.read_documents(args.lenient_length.unwrap_or_default().into())?;

    let mut context = EntryContext {
        instruments: read_items_from_tomls(
            config
                .sibling_file(INSTRUMENTS_FILENAME)
                .into_iter()
                .chain(args.instruments_files),
        )?,
        operators: &config.operators,
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

fn build_entry(
    record: &AdifRecord,
    qsl_status: QslStatus,
    context: &mut EntryContext,
) -> Result<QslCardEntry> {
    let qso_record = QsoRecord::new(record, UtcOffset::UTC)?;
    let qso_exchanges = QsoExchanges::new(record);

    let mut instrument_key = context.default_instrument;
    let comment = get_optional_field(record, "COMMENT").unwrap_or_default();
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
    let power = get_optional_field(record, "TX_PWR")
        .and_then(|p| match p.parse() {
            Ok(power) => Some(power),
            Err(e) => {
                warn!("invalid TX_PWR {p}: {e}");
                None
            }
        })
        .filter(is_valid_power)
        .or(context.default_power.filter(is_valid_power))
        .or(instrument
            .and_then(|i| i.default_power)
            .filter(is_valid_power));

    let operator_callsign = get_optional_field(record, "OPERATOR");
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

/// Checks whether the power value is valid; zero is treated as missing.
fn is_valid_power(power: &f64) -> bool {
    *power != 0.0
}

fn build_station(
    record: &AdifRecord,
    subdivisions: Option<&mut SubdivisionResolver>,
) -> QslStation {
    let grid =
        get_optional_field(record, "MY_GRIDSQUARE").and_then(|g| match g.parse::<GridLocator>() {
            Ok(grid) => Some(grid),
            Err(e) => {
                warn!("invalid MY_GRIDSQUARE {g}: {e}");
                None
            }
        });

    let state = compact_field(record, "MY_STATE");
    let dxcc = get_optional_field(record, "MY_DXCC").and_then(|d| d.parse::<u32>().ok());
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

fn compact_field(record: &AdifRecord, name: &str) -> Option<CompactString> {
    get_optional_field(record, name).map(|v| v.to_compact_string())
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
    let script_table: LuaTable = lua
        .load(script_text)
        .set_name(format!("@{}", script_path.display()))
        .eval()?;
    let generate: LuaFunction = script_table.get("generate")?;
    let processed_value: LuaValue = generate.call((script_args, entries))?;

    Ok(lua_to_json(processed_value)?)
}
