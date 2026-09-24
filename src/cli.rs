use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::qcgen;

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
}
