use std::{convert::Infallible, path::PathBuf, str::FromStr};

use adif_reader::LengthMode;
use clap::{Args, ValueEnum};
use compact_str::{CompactString, ToCompactString};
use time::{Date, error::Parse as TimeParseError, macros::format_description};

/// Generates JSON data for QSL cards.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Processor script file.
    pub script_path: PathBuf,

    /// Read QSOs from ADIF file instead of Wavelog.
    #[arg(long, value_name = "FILE")]
    pub adif: Option<PathBuf>,

    /// Fetch QSOs only from specified Wavelog station locations.
    #[arg(
        long,
        conflicts_with = "adif",
        value_delimiter = ',',
        value_name = "ID"
    )]
    pub station_id: Vec<u64>,

    /// Fetch QSOs on or after the date (YYYY-MM-DD).
    #[arg(long, conflicts_with = "adif", value_parser = parse_date, value_name = "DATE")]
    pub qso_since: Option<Date>,

    /// Fetch QSOs on or before the date (YYYY-MM-DD).
    #[arg(long, conflicts_with = "adif", value_parser = parse_date, value_name = "DATE")]
    pub qso_until: Option<Date>,

    /// Do not resolve subdivision names via Wavelog catalog.
    #[arg(long)]
    pub no_state_names: bool,

    /// Process all QSOs regardless of QSL_SENT.
    /// By default, only QSOs with QSL_SENT of R (requested) or Q (queued) are processed.
    #[arg(long)]
    pub all_qsos: bool,

    /// Enable lenient length count for ADI file.
    /// Pedantic ADI file must not contain non-ASCII characters.
    #[arg(short, long = "lenient")]
    pub lenient_length: Option<LenientMode>,

    /// Specify additional instruments definition file.
    /// instruments.toml next to config file is always loaded first if exists.
    #[arg(short, long = "instruments")]
    pub instruments_files: Vec<PathBuf>,

    /// Specify arguments passed to script.
    #[arg(short = 'A', long = "args")]
    pub script_args: Vec<ScriptArg>,

    /// Specify default instrument.
    #[arg(short = 'I', long)]
    pub instrument: Option<String>,

    /// Specify default power.
    /// It overrides default instrument's value.
    #[arg(short = 'P', long)]
    pub power: Option<f64>,
}

fn parse_date(s: &str) -> Result<Date, TimeParseError> {
    Date::parse(s, format_description!("[year]-[month]-[day]"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum LenientMode {
    /// Count by bytes.
    #[default]
    Bytes,

    /// Count by codepoints.
    Codepoints,

    /// Count by grapheme clusters.
    Graphemes,
}

impl From<LenientMode> for LengthMode {
    fn from(value: LenientMode) -> Self {
        match value {
            LenientMode::Bytes => LengthMode::Bytes,
            LenientMode::Codepoints => LengthMode::Codepoints,
            LenientMode::Graphemes => LengthMode::Graphemes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptArg(pub CompactString, pub Option<CompactString>);

impl FromStr for ScriptArg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            Some((k, v)) => Ok(ScriptArg(
                k.to_compact_string(),
                Some(v.to_compact_string()),
            )),
            None => Ok(ScriptArg(s.to_compact_string(), None)),
        }
    }
}
