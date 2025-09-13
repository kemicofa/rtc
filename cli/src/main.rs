use anyhow::{ Ok, Result };
use clap::Parser;
use common::tracing::init_tracing;
use gcp_traces::trace::TracesAPI;

use crate::cli::{ Cli, Commands };
use crate::ingest::ingest;

mod cli;
mod ingest;
mod env;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    // let traces = TracesAPI::new().await?;
    // traces.get_trace("shine-api-staging".into(), "a54bb62c354c5078cc81f0f72f334e6d".into()).await?;

    match cli.command {
        Commands::Ingest => {
            ingest().await?;
        }
    }

    Ok(())
}
