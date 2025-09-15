use anyhow::{ Ok, Result };
use clap::Parser;
use common::tracing::init_tracing;

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
    dotenvy::dotenv().expect("Unable to load environment variables");
    init_tracing();
    let cli = Cli::parse();

    let config = match cli.command {
        Commands::Ingest => Config::default(),
        Commands::Demo => Config::new(GraphEngine::Falkor, LogEngine::Fake),
    };

    let logs_to_graph = build_dependencies(config).await?;

    logs_to_graph.run().await?;

    Ok(())
}
