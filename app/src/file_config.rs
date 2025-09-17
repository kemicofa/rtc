use std::fs;

use anyhow::{ Context, Result };
use serde::Deserialize;

use crate::config::{ GraphEngine, HttpConfig, LogEngine };

#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    pub schema_version: Option<u32>,
    pub log_engine: Option<LogEngine>,
    pub graph_engine: Option<GraphEngine>,
    pub http: Option<HttpConfig>,
}

pub fn load_file_config(path: std::path::PathBuf) -> Result<FileConfig> {
    let data = fs
        ::read_to_string(path.clone())
        .with_context(|| format!("Failed reading config file: {}", path.clone().display()))?;

    let cfg: FileConfig = toml
        ::from_str(&data)
        .with_context(|| format!("Failed parsing TOML in {}", path.display()))?;

    Ok(cfg)
}
