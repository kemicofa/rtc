use clap::{ Parser, Subcommand, Args };

/// A tiny demo CLI with multiple commands (and a nested group).
#[derive(Debug, Parser)]
#[command(name = "rtc", version, about = "Real Time Cartographer")]
#[command(propagate_version = true)] // lets subcommands inherit --version
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Ingest,
    Demo,
}
