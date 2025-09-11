// src/lib.rs
use std::{ collections::HashMap, num::NonZeroU8 };
use anyhow::Result;
use falkordb::{ FalkorClientBuilder, FalkorConnectionInfo, FalkorAsyncClient };

use crate::service::Service;

pub mod service;

pub struct ServiceGraphBuilder {
    client: FalkorAsyncClient,
    graph: String,
}

impl ServiceGraphBuilder {
    /// Build a client from a URL like:
    ///  - "redis://127.0.0.1:6379" (plain)
    ///  - "rediss://host:6380" (TLS, use tokio-rustls or tokio-native-tls feature)
    ///  - "falkor://host:6379" (accepted by the client)
    pub async fn new(url: String, graph: String, pool: NonZeroU8) -> Result<Self> {
        let info: FalkorConnectionInfo = url.try_into()?;
        let client = FalkorClientBuilder::new_async()
            .with_connection_info(info)
            .with_num_connections(pool) // small connection pool for concurrency
            .build().await?;
        Ok(Self { client, graph: graph.to_string() })
    }

    pub async fn add_service_node(&self, service: Service) -> Result<()> {
        let cypher =
            r#"
            MERGE (:Service { id:$id, name: $name })
            ON CREATE SET s.name = $name, s.email = $email
            ON MATCH SET s.name = $name, s.email = $email
        "#;

        Ok(())
    }

    /// Example: parameterized insert with timeout
    pub async fn insert_user(&self, id: i64, name: &str, email: &str) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("id".into(), id.to_string());
        params.insert("name".into(), name.to_string());
        params.insert("email".into(), email.to_string());

        let cypher = "CREATE (:User {id:$id, name:$name, email:$email})";

        self.client
            .select_graph(&self.graph)
            .query(cypher)
            .with_params(&params)
            .with_timeout(5_000) // ms
            .execute().await?;
        Ok(())
    }
}
