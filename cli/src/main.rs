use anyhow::{ Ok, Result };
use clap::Parser;
use common::tracing::init_tracing;

use crate::cli::{ Cli, Commands };
use crate::ingest::ingest;

mod cli;
mod ingest;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest => {
            ingest().await?;
        }
    }

    Ok(())
}
