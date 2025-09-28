use std::collections::{ HashMap, HashSet };

use serde::{ Deserialize, Serialize };
use crate::hash;

pub type ServiceName = String;

pub type HttpMethod = String;
pub type HttpPath = String;

pub type OperationId = String;
pub type ServiceOperationId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Operation {
    Http {
        method: HttpMethod,
        path: HttpPath,
    },
}

impl Operation {
    fn get_id(&self) -> String {
        match self {
            Operation::Http { method, path } =>
                format!("http_{}_{}", method.trim().to_lowercase(), path.trim().to_lowercase()),
        }
    }

    pub fn get_label(&self) -> String {
        match self {
            Operation::Http { method, path } =>
                format!("{} {}", method.trim().to_uppercase(), path.trim().to_lowercase()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceNode {
    pub name: ServiceName,
    pub operations: HashMap<ServiceOperationId, Operation>,
    pub invokes: HashMap<ServiceName, HashSet<ServiceOperationId>>,
}

impl ServiceNode {
    fn new(name: String) -> Self {
        Self {
            name,
            operations: Default::default(),
            invokes: Default::default(),
        }
    }

    /// Generates a unique id for an operation across all services
    fn get_service_operation_id(&self, operation: &Operation) -> String {
        // Generate an operation id that is uniquely associated to the service.
        let raw_operation_id = format!("{}_{}", self.name.clone(), operation.get_id());
        let service_operation_id = hash!(raw_operation_id.as_str());
        service_operation_id
    }

    fn add_operation(&mut self, operation: Operation) {
        let operation_id = self.get_service_operation_id(&operation);
        self.operations.insert(operation_id.clone(), operation);
    }

    fn add_target(&mut self, name: ServiceName, operation: Operation) {
        let service_operation_id = self.get_service_operation_id(&operation);

        self.invokes
            .entry(name)
            .and_modify(|target| {
                target.insert(service_operation_id.clone());
            })
            .or_insert(HashSet::from_iter([service_operation_id]));
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceNodeGraph {
    pub services: HashMap<ServiceName, ServiceNode>,
}

impl Default for ServiceNodeGraph {
    fn default() -> Self {
        Self {
            services: Default::default(),
        }
    }
}

impl ServiceNodeGraph {
    pub fn add_service(&mut self, name: ServiceName) {
        self.services.insert(name.clone(), ServiceNode::new(name.clone()));
    }

    /// Method that adds an operation to an existing ServiceNode.
    /// If the ServiceNode does not exist, then it'll create one on the fly and add the operation.
    pub fn add_operation_to_service(&mut self, name: ServiceName, operation: Operation) {
        self.services
            .entry(name.clone())
            .and_modify(|service| {
                service.add_operation(operation.clone());
            })
            .or_insert(ServiceNode::new(name.clone()))
            .add_operation(operation);
    }

    pub fn add_target_to_service(
        &mut self,
        from: ServiceName,
        to: ServiceName,
        using_operation: Operation
    ) {
        self.services
            .entry(from.clone())
            .and_modify(|service| {
                service.add_target(to.clone(), using_operation.clone());
            })
            .or_insert(ServiceNode::new(from.clone()))
            .add_target(to, using_operation);
    }
}

/**
 * @todo Tests incomplete
 */
#[cfg(test)]
mod test {
    use crate::service_node_graph::{ Operation, ServiceNodeGraph };

    #[test]
    fn should_be_able_to_add_a_service() {
        let mut graph = ServiceNodeGraph::default();
        graph.add_service("users-service".into());
        let json_string = serde_json::to_string(&graph).expect("Failed to serialize graph");

        assert_eq!(
            json_string,
            "{\"services\":{\"users-service\":{\"name\":\"users-service\",\"operations\":{},\"invokes\":{}}}}".to_string()
        );
    }

    #[test]
    fn should_be_able_to_add_an_operation() {
        let mut graph = ServiceNodeGraph::default();
        let service_name: String = "users-service".into();
        graph.add_service(service_name.clone());
        graph.add_operation_to_service(service_name, Operation::Http {
            method: "post".into(),
            path: "/users".into(),
        });
        let json_string = serde_json::to_string(&graph).expect("Failed to serialize graph");

        assert_eq!(
            json_string,
            "{\"services\":{\"users-service\":{\"name\":\"users-service\",\"operations\":{\"13e8e5c0b41a85cfcf1f7b34ab159aa29be20f6c7631fcaff120d07c923322aa\":{\"Http\":[\"post\",\"/users\"]}},\"invokes\":{}}}}".to_string()
        );
    }

    #[test]
    fn should_be_able_to_add_multiple_operations() {
        let mut graph = ServiceNodeGraph::default();
        let service_name: String = "users-service".into();
        graph.add_service(service_name.clone());
        graph.add_operation_to_service(service_name.clone(), Operation::Http {
            method: "post".into(),
            path: "/users".into(),
        });
        graph.add_operation_to_service(service_name, Operation::Http {
            method: "get".into(),
            path: "/users/{user_id}".into(),
        });
        let json_string = serde_json::to_string(&graph).expect("Failed to serialize graph");

        assert_eq!(
            json_string,
            "{\"services\":{\"users-service\":{\"name\":\"users-service\",\"operations\":{\"13e8e5c0b41a85cfcf1f7b34ab159aa29be20f6c7631fcaff120d07c923322aa\":{\"Http\":[\"post\",\"/users\"]},\"8c4475ac37e66057f4a7304fd76a46fe7f0313b7626b6123cae82f024c728fba\":{\"Http\":[\"get\",\"/users/{user_id}\"]}},\"invokes\":{}}}}".to_string()
        );
    }

    #[test]
    fn should_be_able_to_add_a_target_service_even_though_it_does_not_exist() {
        let mut graph = ServiceNodeGraph::default();
        let service_name_book: String = "books-service".into();
        let service_name_user: String = "users-service".into();

        graph.add_service(service_name_user.clone());
        graph.add_target_to_service(service_name_user, service_name_book, Operation::Http {
            method: "post".into(),
            path: "/books".into(),
        });

        let json_string = serde_json::to_string(&graph).expect("Failed to serialize graph");

        assert_eq!(
            json_string,
            "{\"services\":{\"users-service\":{\"name\":\"users-service\",\"operations\":{},\"invokes\":{\"books-service\":[\"3dff280d8c0d7beed2f82e8f2ffc2bcf65608fd62579ba9047137f9ec884e984\"]}}}}".to_string()
        );
    }

    #[test]
    fn should_be_able_to_add_multiple_operations_to_a_service() {
        let mut graph = ServiceNodeGraph::default();
        let service_name: String = "users-service".into();
        graph.add_service(service_name.clone());
        graph.add_operation_to_service(service_name.clone(), Operation::Http {
            method: "post".into(),
            path: "/users".into(),
        });
        graph.add_operation_to_service(service_name, Operation::Http {
            method: "get".into(),
            path: "/users/{user_id}".into(),
        });
        let json_string = serde_json::to_string(&graph).expect("Failed to serialize graph");

        assert_eq!(
            json_string,
            "{\"services\":{\"users-service\":{\"name\":\"users-service\",\"operations\":{\"13e8e5c0b41a85cfcf1f7b34ab159aa29be20f6c7631fcaff120d07c923322aa\":{\"Http\":[\"post\",\"/users\"]},\"8c4475ac37e66057f4a7304fd76a46fe7f0313b7626b6123cae82f024c728fba\":{\"Http\":[\"get\",\"/users/{user_id}\"]}},\"invokes\":{}}}}".to_string()
        );
    }
}
