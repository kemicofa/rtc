use std::path::PathBuf;

use anyhow::{ Ok, Result, bail };
use clap::Parser;
use common::tracing::init_tracing;

use crate::bootstrap::build_dependencies;
use crate::cli::{ Cli, Commands };
use crate::config::{ Config, LogEngine };
use crate::file_config::load_file_config;

mod cli;
mod bootstrap;
mod config;
mod fake_service_log;
mod file_config;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    let config = match cli.command {
        Commands::Run { config } => {
            let path: PathBuf = if config.is_some() {
                config.unwrap()
            } else {
                // TODO: make this OS agnostic
                "./rtc.toml".into()
            };
            let cfg = load_file_config(path)?;

            if cfg.graph_engine.is_none() {
                bail!("A graph engine must be specified in the config file");
            }

            if cfg.log_engine.is_none() {
                bail!("A log engine must be specified in the config file");
            }

            Config::new(cfg.graph_engine.unwrap(), cfg.log_engine.unwrap(), cfg.http)
        }
        Commands::Demo => {
            // TODO: make this OS agnostic
            let cfg = load_file_config("./rtc.demo.toml".into())?;

            if cfg.graph_engine.is_none() {
                bail!("A graph engine should be specified in the rtc.demo.toml");
            }

            Config::new(cfg.graph_engine.unwrap(), LogEngine::Fake, None)
        }
    };

    let logs_to_graph = build_dependencies(config).await?;

    logs_to_graph.run().await?;

    Ok(())
}
