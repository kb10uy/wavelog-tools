use clap::{Parser, Subcommand};

use crate::qcgen;

/// Tools using Wavelog API.
#[derive(Debug, Clone, Parser)]
#[command(version, author, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Qcgen(qcgen::Arguments),
}
