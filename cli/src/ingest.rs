use std::num::NonZeroU8;

use anyhow::{ Ok, Result };
use gcp_log_ingestor::GCPLogIngestor;
use service_graph_builder::ServiceGraphBuilder;
use tracing::{ debug, info, warn };

pub async fn ingest() -> Result<()> {
    info!("Initiating gcp-log-ingestor");

    let project_id = std::env::var("PROJECT_ID").expect("Set PROJECT_ID");
    let log_filter = std::env::var("LOG_FILTER").expect("Set LOG_FILTER");
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

    info!("Connecting to graph database");
    let service_graph = ServiceGraphBuilder::new(database_url, graph_name, pool).await?;

    info!("Building logging service");
    let gcp_log_ingestor = GCPLogIngestor::new(project_id, page_size, max_pages, log_filter).await?;

    // TODO: make a proper return for this function
    let res = gcp_log_ingestor.run(async move |e| {
        debug!("{:?}", e);

        let trace = e.trace;
        let insert_id = e.insert_id;

        if insert_id.is_empty() {
            warn!("Detected a log entry without an insert_id");
        }

        if trace.is_empty() {
            warn!("Detected a log entry without a trace");
        }

        if e.http_request.is_none() {
            debug!("Skipping log entry as it does not have http request information");
            return;
        }

        let http_request = e.http_request.unwrap();

        let request_url = http_request.request_url;
        let request_method = http_request.request_method;
        let status = http_request.status;

        // let service = Service {};

        // service_graph.add_service_node();
    }).await?;

    Ok(())
}
