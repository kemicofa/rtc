use std::num::{ NonZeroU8 };

use serde::Deserialize;

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where D: serde::Deserializer<'de>
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}

#[derive(Debug, Deserialize, Default)]
pub struct HttpRequestPaths {
    pub custom_normalize_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct HttpConfig {
    pub request_paths: HttpRequestPaths,
}

#[derive(Debug, Deserialize)]
pub enum LogEngine {
    #[serde(rename = "gcp")] GCP {
        project_id: String,
        page_size: Option<i32>,
        max_pages: Option<i32>,
        #[serde(deserialize_with = "empty_string_as_none")]
        custom_log_filter: Option<String>,
    },
    Fake,
}

#[derive(Debug, Deserialize)]
pub enum GraphEngine {
    #[serde(rename = "falkor")] Falkor {
        database_url: String,
        max_pool: Option<NonZeroU8>,
        graph_name: String,
    },
}

pub struct Config {
    pub graph_engine: GraphEngine,
    pub log_engine: LogEngine,
    pub http_config: Option<HttpConfig>,
}

impl Config {
    pub fn new(
        graph_engine: GraphEngine,
        log_engine: LogEngine,
        http_config: Option<HttpConfig>
    ) -> Self {
        Self {
            graph_engine,
            log_engine,
            http_config,
        }
    }
}
