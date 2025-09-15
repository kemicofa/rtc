use anyhow::{ Ok, Result, bail };
use clap::Parser;
use common::tracing::init_tracing;
use dotenvy::{ from_filename_override };

use crate::bootstrap::build_dependencies;
use crate::cli::{ Cli, Commands };
use crate::config::{ Config, GraphEngine, LogEngine };

mod cli;
mod env;
mod bootstrap;
mod config;
mod fake_service_log;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        bail!("Unable to load environment variables: {}", e);
    }
    init_tracing();

    let cli = Cli::parse();

    let config = match cli.command {
        Commands::Ingest => { Config::default() }
        Commands::Demo => {
            if let Err(e) = from_filename_override(".env.demo") {
                bail!("Failed to read .env.demo: {}", e);
            }
            Config::new(GraphEngine::Falkor, LogEngine::Fake)
        }
    };

    let logs_to_graph = build_dependencies(config).await?;

    logs_to_graph.run().await?;

    Ok(())
}
