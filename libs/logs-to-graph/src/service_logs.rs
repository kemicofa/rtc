use async_trait::async_trait;
use anyhow::Result;
use tokio::sync::mpsc::Sender;

use crate::service_node_graph::ServiceNodeGraph;

#[async_trait]
pub trait ServiceLogs: Send + Sync {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()>;
}
