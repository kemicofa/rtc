use std::sync::Arc;

use anyhow::{ Ok, Result };
use common::types::BMArc;
use service_logs::ServiceLogs;
use tokio::{ sync::{ mpsc }, task };
use tracing::error;

use crate::{ service_graph::ServiceGraph, service_node_graph::ServiceNodeGraph };

pub mod service_graph;
pub mod service_logs;
pub mod service_node_graph;
mod macros;

/// Max number of messages in queue before backpressure is applied
const CHANNEL_BUFFER_SIZE: usize = 32;

pub struct LogsToGraph {
    service_graph: BMArc<dyn ServiceGraph>,
    service_logs: BMArc<dyn ServiceLogs>,
}

impl LogsToGraph {
    pub fn new(
        service_graph: BMArc<dyn ServiceGraph>,
        service_logs: BMArc<dyn ServiceLogs>
    ) -> Self {
        Self {
            service_graph,
            service_logs,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<ServiceNodeGraph>(CHANNEL_BUFFER_SIZE);

        let service_logs = Arc::clone(&self.service_logs);
        let service_graph = Arc::clone(&self.service_graph);

        let producer = task::spawn(async move { service_logs.lock().await.run(sender).await });
        let consumer = task::spawn(async move { service_graph.lock().await.run(receiver).await });

        let _ = tokio::join!(producer, consumer);

        Ok(())
    }
}
