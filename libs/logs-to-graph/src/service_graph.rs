use crate::service_node_graph::ServiceNodeGraph;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[async_trait]
pub trait ServiceGraph: Send + Sync {
    async fn run(&mut self, mut receiver: Receiver<ServiceNodeGraph>) -> Result<()>;
}
