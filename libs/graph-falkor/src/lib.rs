use std::{ collections::HashMap, num::NonZeroU8 };
use anyhow::{ Ok, Result };
use async_trait::async_trait;
use falkordb::{ FalkorClientBuilder, FalkorConnectionInfo, FalkorAsyncClient };
use logs_to_graph::{
    service_graph::ServiceGraph,
    service_node_graph::{ Operation, ServiceName, ServiceNodeGraph, ServiceOperationId },
};
use tokio::sync::mpsc::Receiver;
use tracing::{ debug, info, warn };

use crate::consts::{
    UPSERT_OPERATION_CYPHER,
    UPSERT_SERVICE_NODE_CYPHER,
    UPSERT_SERVICE_NODE_TO_OPERATION_CYPHER,
    UPSERT_SERVICE_TO_SERVICE_OPERATION_RELATION,
};

mod consts;
mod macros;

pub struct GraphFalkor {
    client: FalkorAsyncClient,
    graph: String,
}

impl GraphFalkor {
    pub async fn new(url: String, graph: String, max_pool: NonZeroU8) -> Result<Self> {
        let info: FalkorConnectionInfo = url.try_into()?;
        let client = FalkorClientBuilder::new_async()
            .with_connection_info(info)
            .with_num_connections(max_pool) // small connection pool for concurrency
            .build().await?;
        Ok(Self { client, graph: graph.to_string() })
    }

    async fn upsert_service_node(&self, name: String) -> Result<()> {
        let params: HashMap<String, String> = HashMap::from_iter([
            ("name".into(), stringy!(name.clone())),
            ("id".into(), stringy!(name.clone())),
        ]);
        let mut graph = self.client.select_graph(self.graph.clone());
        let res = graph.query(UPSERT_SERVICE_NODE_CYPHER).with_params(&params).execute().await?;

        if res.get_nodes_created().is_some_and(|count| count > 0) {
            info!("Created new service node: {}", name);
        }
        Ok(())
    }

    async fn upsert_service_node_operation(
        &self,
        name: String,
        service_operation_id: String,
        operation: Operation
    ) -> Result<()> {
        let operation_label = operation.get_label();
        let operation_params: HashMap<String, String> = HashMap::from_iter([
            ("label".into(), stringy!(operation_label.clone())),
            ("id".into(), stringy!(service_operation_id.clone())),
        ]);

        let mut graph = self.client.select_graph(self.graph.clone());
        let upsert_operation_res = graph
            .query(UPSERT_OPERATION_CYPHER)
            .with_params(&operation_params)
            .execute().await?;

        if upsert_operation_res.get_nodes_created().is_some_and(|count| count > 0) {
            info!("Created new operation node: {:?}", operation);
        }

        let service_node_to_operation_relation_params: HashMap<String, String> = HashMap::from_iter(
            [
                ("id".into(), stringy!(service_operation_id)),
                ("name".into(), stringy!(name.clone())),
            ]
        );

        let upsert_service_node_to_operation_res = graph
            .query(UPSERT_SERVICE_NODE_TO_OPERATION_CYPHER)
            .with_params(&service_node_to_operation_relation_params)
            .execute().await?;

        if upsert_service_node_to_operation_res.get_nodes_created().is_some_and(|count| count > 0) {
            info!("Created new service to operation node: {} -> {:?}", name, operation);
        }

        Ok(())
    }

    async fn upsert_service_to_service_operation_relation(
        &self,
        from_service_name: String,
        to_service_operation_id: String
    ) -> Result<()> {
        let params: HashMap<String, String> = HashMap::from_iter([
            ("name".into(), stringy!(from_service_name)),
            ("id".into(), stringy!(to_service_operation_id)),
        ]);

        let mut graph = self.client.select_graph(self.graph.clone());
        let res = graph
            .query(UPSERT_SERVICE_TO_SERVICE_OPERATION_RELATION)
            .with_params(&params)
            .execute().await?;

        if res.get_nodes_created().is_some_and(|count| count > 0) {
            info!(
                "Created new service to service operation: {} -> {}",
                from_service_name,
                to_service_operation_id
            );
        }

        Ok(())
    }

    async fn process(&mut self, service_node_graph: ServiceNodeGraph) -> Result<()> {
        let mut service_to_service_relations: Vec<(ServiceName, ServiceOperationId)> = vec![];
        debug!("Processing service node graph");
        for (service_name, service_node) in service_node_graph.services.iter() {
            let upsert_service_node_res = self.upsert_service_node(service_name.clone()).await;

            if upsert_service_node_res.is_err() {
                let e = upsert_service_node_res.unwrap_err();
                warn!("Failed to upsert service node: {}", e);
                return Ok(());
            }

            for (service_operation_id, operation) in service_node.operations.iter() {
                let upsert_service_node_operation_result = self.upsert_service_node_operation(
                    service_name.clone(),
                    service_operation_id.clone(),
                    operation.clone()
                ).await;

                if upsert_service_node_operation_result.is_err() {
                    let e = upsert_service_node_operation_result.unwrap_err();
                    warn!("Failed to upsert service node operation: {}", e);
                    return Ok(());
                }
            }

            for (_, to_service_operation_ids) in service_node.invokes.iter() {
                for to_service_operation_id in to_service_operation_ids.iter() {
                    service_to_service_relations.push((
                        service_name.clone(),
                        to_service_operation_id.clone(),
                    ));
                }
            }
        }

        for (from_service_name, to_service_operation_id) in service_to_service_relations {
            let upsert_service_to_service_operation_relation_res =
                self.upsert_service_to_service_operation_relation(
                    from_service_name,
                    to_service_operation_id
                ).await;

            if upsert_service_to_service_operation_relation_res.is_err() {
                let e = upsert_service_to_service_operation_relation_res.unwrap_err();
                warn!("Failed to create an INVOKES relation between service node and a target service node operation: {}", e);
                return Ok(());
            }
        }
        debug!("Done processing service_node_graph");
        Ok(())
    }
}

#[async_trait]
impl ServiceGraph for GraphFalkor {
    async fn run(&mut self, mut receiver: Receiver<ServiceNodeGraph>) -> Result<()> {
        while let Some(service_node_graph) = receiver.recv().await {
            let process_res = self.process(service_node_graph).await;

            if process_res.is_err() {
                let e = process_res.unwrap_err();
                warn!("Failed to process ServiceNodeGraph: {}", e);
            }
        }

        Ok(())
    }
}
