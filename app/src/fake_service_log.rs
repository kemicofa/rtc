use std::collections::{ HashMap, HashSet };

use async_trait::async_trait;
use logs_to_graph::{
    service_logs::ServiceLogs,
    service_node_graph::{ Operation, ServiceNode, ServiceNodeGraph },
};
use anyhow::{ Result, Ok };
use tokio::sync::mpsc::Sender;

pub struct FakeServiceLog {}

impl Default for FakeServiceLog {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl ServiceLogs for FakeServiceLog {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()> {
        sender.send(ServiceNodeGraph {
            services: HashMap::from_iter([
                (
                    "web-app".into(),
                    ServiceNode {
                        name: "web-app".into(),
                        operations: HashMap::from_iter([]),
                        invokes: HashMap::from_iter([
                            (
                                "graphql-service".into(),
                                HashSet::from_iter(["graphql-service_http_post_/".into()]),
                            ),
                        ]),
                    },
                ),
                (
                    "graphql-service".into(),
                    ServiceNode {
                        name: "graphql-service".into(),
                        operations: HashMap::from_iter([
                            (
                                "graphql-service_http_post_/".into(),
                                Operation::Http("POST".into(), "/".into()),
                            ),
                        ]),
                        invokes: HashMap::from_iter([
                            (
                                "books-service".into(),
                                HashSet::from_iter([
                                    "books-service_http_post_/books".into(),
                                    "books-service_http_get_/books/{books_id}".into(),
                                    "books-service_http_post_/books/{books_id}/chapters".into(),
                                    "books-service_http_get_/books/{books_id}/chapters/{chapter_id}".into(),
                                ]),
                            ),
                            (
                                "users-service".into(),
                                HashSet::from_iter([
                                    "users-service_http_post_/users".into(),
                                    "users-service_http_get_/users/{users_id}".into(),
                                ]),
                            ),
                            (
                                "auth-service".into(),
                                HashSet::from_iter([
                                    "auth-service_http_post_/login".into(),
                                    "auth-service_http_post_/logout".into(),
                                ]),
                            ),
                        ]),
                    },
                ),
                (
                    "users-service".into(),
                    ServiceNode {
                        name: "users-service".into(),
                        operations: HashMap::from_iter([
                            (
                                "users-service_http_post_/users".into(),
                                Operation::Http("POST".into(), "/users".into()),
                            ),
                            (
                                "users-service_http_get_/users/{users_id}".into(),
                                Operation::Http("GET".into(), "/users/{users_id}".into()),
                            ),
                        ]),
                        invokes: HashMap::from_iter([]),
                    },
                ),
                (
                    "books-service".into(),
                    ServiceNode {
                        name: "books-service".into(),
                        operations: HashMap::from_iter([
                            (
                                "books-service_http_post_/books".into(),
                                Operation::Http("POST".into(), "/books".into()),
                            ),
                            (
                                "books-service_http_get_/books/{books_id}".into(),
                                Operation::Http("GET".into(), "/books/{books_id}".into()),
                            ),
                            (
                                "books-service_http_post_/books/{books_id}/chapters".into(),
                                Operation::Http("POST".into(), "/books/{books_id}/chapters".into()),
                            ),
                            (
                                "books-service_http_get_/books/{books_id}/chapters/{chapter_id}".into(),
                                Operation::Http(
                                    "GET".into(),
                                    "/books/{books_id}/chapters/{chapters_id}".into()
                                ),
                            ),
                        ]),
                        invokes: HashMap::from_iter([]),
                    },
                ),
                (
                    "auth-service".into(),
                    ServiceNode {
                        name: "auth-service".into(),
                        operations: HashMap::from_iter([
                            (
                                "auth-service_http_post_/login".into(),
                                Operation::Http("POST".into(), "/login".into()),
                            ),
                            (
                                "auth-service_http_post_/logout".into(),
                                Operation::Http("POST".into(), "/logout".into()),
                            ),
                        ]),
                        invokes: HashMap::from_iter([
                            (
                                "users-service".into(),
                                HashSet::from_iter([
                                    "users-service_http_get_/users/{users_id}".into(),
                                ]),
                            ),
                        ]),
                    },
                ),
            ]),
        }).await?;
        Ok(())
    }
}
