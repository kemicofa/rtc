use std::num::NonZeroU8;

pub struct GCPLoggingEnv {
    pub project_id: String,
    pub log_filter: Option<String>,
    pub page_size: i32,
    pub max_pages: i32,
    pub custom_path_regex: Option<String>,
}

pub struct FalkorEnv {
    pub graph_name: String,
    pub database_url: String,
    pub pool: NonZeroU8,
}

pub fn load_gcp_log_env() -> GCPLoggingEnv {
    let project_id = std::env::var("PROJECT_ID").expect("Set PROJECT_ID");
    let log_filter: Option<String> = match std::env::var("LOG_FILTER").unwrap_or("".into()).trim() {
        "" => None,
        val => Some(val.into()),
    };
    let page_size = std::env
        ::var("PAGE_SIZE")
        .expect("Set PAGE_SIZE")
        .parse::<i32>()
        .expect("Expected PAGE_SIZE to be a number");
    let max_pages = std::env
        ::var("MAX_PAGES")
        .expect("Set MAX_PAGES")
        .parse::<i32>()
        .expect("Expected MAX_PAGES to be a number");
    let custom_path_regex: Option<String> = match
        std::env::var("CUSTOM_PATH_REGEX").unwrap_or("".into()).trim()
    {
        "" => None,
        val => Some(val.into()),
    };

    GCPLoggingEnv { project_id, log_filter, page_size, max_pages, custom_path_regex }
}

pub fn load_falkor_env() -> FalkorEnv {
    let graph_name = std::env::var("GRAPH_NAME").expect("Set GRAPH_NAME");
    let database_url = std::env::var("DATABASE_URL").expect("Set DATABASE_URL");
    let pool: NonZeroU8 = std::env
        ::var("DATABASE_MAX_CONNECTION_POOL_COUNT")
        .expect("Set DATABASE_MAX_CONNECTION_POOL_COUNT")
        .parse::<NonZeroU8>()
        .expect("DATABASE_MAX_CONNECTION_POOL_COUNT must be a non zero u8");

    FalkorEnv { graph_name, database_url, pool }
}
