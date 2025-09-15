use common::{ bmarc, types::BMArc };
use gcp::gcp_service_log::GCPServiceLogs;
use graph_falkor::GraphFalkor;
use anyhow::{ Ok, Result };
use logs_to_graph::{ LogsToGraph, service_graph::ServiceGraph, service_logs::ServiceLogs };

use crate::{
    config::{ Config, GraphEngine, LogEngine },
    env::{ load_falkor_env, load_gcp_log_env },
    fake_service_log::FakeServiceLog,
};

pub async fn build_dependencies(config: Config) -> Result<LogsToGraph> {
    let service_graph: BMArc<dyn ServiceGraph> = match config.graph_engine {
        GraphEngine::Falkor => {
            let env = load_falkor_env();
            let graph = GraphFalkor::new(env.database_url, env.graph_name, env.pool).await?;
            bmarc!(graph)
        }
        GraphEngine::EmitOnly => todo!(),
    };

    let service_logs: BMArc<dyn ServiceLogs> = match config.log_engine {
        LogEngine::GCP => {
            let env = load_gcp_log_env();
            let service_logs = GCPServiceLogs::new(
                env.project_id,
                env.page_size,
                env.max_pages,
                env.log_filter,
                env.custom_path_regex
            ).await?;

            bmarc!(service_logs)
        }
        LogEngine::Fake => {
            let service_logs = FakeServiceLog::default();
            bmarc!(service_logs)
        }
    };

    Ok(LogsToGraph::new(service_graph, service_logs))
}
