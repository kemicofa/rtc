use std::num::NonZeroU8;

pub struct Env {
    pub project_id: String,
    pub log_filter: Option<String>,
    pub graph_name: String,
    pub database_url: String,
    pub page_size: i32,
    pub max_pages: i32,
    pub pool: NonZeroU8,
    pub custom_path_regex: Option<String>,
}

/// If a required env is not set it will stop the application
pub fn load_env() -> Env {
    dotenvy::dotenv().expect("Unable to load environment variables");

    let project_id = std::env::var("PROJECT_ID").expect("Set PROJECT_ID");
    let log_filter: Option<String> = match std::env::var("LOG_FILTER").unwrap_or("".into()).trim() {
        "" => None,
        val => Some(val.into()),
    };
    let graph_name = std::env::var("GRAPH_NAME").expect("Set GRAPH_NAME");
    let database_url = std::env::var("DATABASE_URL").expect("Set DATABASE_URL");
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
    let pool: NonZeroU8 = std::env
        ::var("DATABASE_MAX_CONNECTION_POOL_COUNT")
        .expect("Set DATABASE_MAX_CONNECTION_POOL_COUNT")
        .parse::<NonZeroU8>()
        .expect("DATABASE_MAX_CONNECTION_POOL_COUNT must be a non zero u8");
    let custom_path_regex: Option<String> = match
        std::env::var("CUSTOM_PATH_REGEX").unwrap_or("".into()).trim()
    {
        "" => None,
        val => Some(val.into()),
    };

    Env {
        project_id,
        log_filter,
        graph_name,
        database_url,
        page_size,
        max_pages,
        pool,
        custom_path_regex,
    }
}
