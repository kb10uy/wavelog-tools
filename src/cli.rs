use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use time::{Date, error::Parse as TimeParseError, macros::format_description};

use crate::{
    export, qcgen,
    wavelog::{QslFilter, QsoQuery},
};

/// Tools using Wavelog API.
#[derive(Debug, Clone, Parser)]
#[command(version, author, about, long_about)]
pub struct Cli {
    /// Specify config file.
    /// Defaults to wavelog-tools/config.toml in the user config directory
    /// ($XDG_CONFIG_HOME or ~/.config on Linux and macOS, %APPDATA% on Windows).
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Qcgen(qcgen::Arguments),
    Export(export::Arguments),
}

/// Conditions for fetching QSOs from Wavelog.
#[derive(Debug, Clone, Args)]
pub struct QsoQueryArgs {
    /// Fetch QSOs only from specified Wavelog station locations.
    #[arg(long, value_delimiter = ',', value_name = "ID")]
    pub station_id: Vec<u64>,

    /// Fetch QSOs only with the worked callsign (exact match).
    #[arg(long, value_name = "CALLSIGN")]
    pub callsign: Option<String>,

    /// Fetch QSOs only on the band (e.g. 20m, or SAT for satellite QSOs).
    #[arg(long)]
    pub band: Option<String>,

    /// Fetch QSOs only in the mode or submode (e.g. SSB, FT8).
    #[arg(long)]
    pub mode: Option<String>,

    /// Fetch QSOs only confirmed via any of the specified types.
    #[arg(long, value_delimiter = ',', value_name = "TYPE")]
    pub qsl_filter: Vec<QslFilter>,

    /// Fetch QSOs only with Wavelog ID greater than the value.
    #[arg(long, default_value_t = 0, value_name = "ID")]
    pub since_id: u64,

    /// Fetch QSOs on or after the date (YYYY-MM-DD).
    #[arg(long, value_parser = parse_date, value_name = "DATE")]
    pub qso_since: Option<Date>,

    /// Fetch QSOs on or before the date (YYYY-MM-DD).
    #[arg(long, value_parser = parse_date, value_name = "DATE")]
    pub qso_until: Option<Date>,
}

impl From<QsoQueryArgs> for QsoQuery {
    fn from(value: QsoQueryArgs) -> Self {
        QsoQuery {
            station_ids: value.station_id,
            callsign: value.callsign,
            band: value.band,
            mode: value.mode,
            qsl_filter: value.qsl_filter,
            since_id: value.since_id,
            qso_since: value.qso_since,
            qso_until: value.qso_until,
        }
    }
}

fn parse_date(s: &str) -> Result<Date, TimeParseError> {
    Date::parse(s, format_description!("[year]-[month]-[day]"))
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verifies_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn rejects_query_with_adif_file() {
        let result = Cli::try_parse_from([
            "wavelog-tools",
            "qcgen",
            "s.lua",
            "--adif",
            "a.adi",
            "--band",
            "20m",
        ]);
        assert!(result.is_err());
    }
}
