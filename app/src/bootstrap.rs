use std::num::NonZeroU8;

use common::{ bmarc, types::BMArc };
use datadog::DatadogServiceLog;
use gcp::gcp_service_log::GCPServiceLogs;
use graph_falkor::GraphFalkor;
use anyhow::{ Ok, Result };
use logs_to_graph::{ LogsToGraph, service_graph::ServiceGraph, service_logs::ServiceLogs };

use crate::{ config::{ Config, GraphEngine, LogEngine }, fake_service_log::FakeServiceLog };

pub async fn build_dependencies(config: Config) -> Result<LogsToGraph> {
    let service_graph: BMArc<dyn ServiceGraph> = match config.graph_engine {
        GraphEngine::Falkor { max_pool, database_url, graph_name } => {
            let revised_max_pool = max_pool.unwrap_or(NonZeroU8::new(1).unwrap());
            let graph = GraphFalkor::new(database_url, graph_name, revised_max_pool).await?;
            bmarc!(graph)
        }
    };

    let service_logs: BMArc<dyn ServiceLogs> = match config.log_engine {
        LogEngine::GCP { project_id, page_size, max_pages, custom_log_filter } => {
            let custom_path_normalize_patterns = config.http_config
                .map(|http| http.request_paths.custom_normalize_patterns)
                .or(Some(vec![]))
                .unwrap();

            let service_logs = GCPServiceLogs::new(
                project_id,
                page_size.unwrap_or(100),
                max_pages.unwrap_or(100),
                custom_log_filter,
                custom_path_normalize_patterns
            ).await?;

            bmarc!(service_logs)
        }
        LogEngine::Datadog { api_key } => {
            let service_logs = DatadogServiceLog::new(api_key)?;
            bmarc!(service_logs)
        }
        LogEngine::Fake => {
            let service_logs = FakeServiceLog::default();
            bmarc!(service_logs)
        }
    };

    Ok(LogsToGraph::new(service_graph, service_logs))
}
