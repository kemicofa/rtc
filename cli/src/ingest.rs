use std::{ collections::{ HashMap, HashSet }, num::NonZeroU8, sync::{ Arc, Mutex } };

use anyhow::{ Ok, Result };
use gcp_log_ingestor::{ GCPLogIngestor, normalize::NormalizedLogEntry };
use service_graph_builder::ServiceGraphBuilder;
use tracing::{ debug, info };

use crate::env::load_env;

type TraceId = String;
type SpanId = String;
type EndpointId = String;
type Metadata = HashMap<TraceId, HashSet<SpanId>>;
type Endpoints = HashMap<EndpointId, EndpointNode>;
#[derive(Debug)]
struct EndpointNode {
    method: String,
    path: String,
    metadata: Metadata,
}

impl EndpointNode {
    pub fn new(method: String, path: String, trace_id: TraceId, span_id: SpanId) -> Self {
        Self {
            method,
            path,
            metadata: HashMap::from_iter([(trace_id, HashSet::from_iter([span_id]))]),
        }
    }

    pub fn add_metadata(&mut self, trace_id: TraceId, span_id: SpanId) {
        self.metadata.entry(trace_id).or_default().insert(span_id);
    }
}

impl PartialEq for EndpointNode {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method && self.path == other.path
    }
}

#[derive(Debug)]
struct ServiceNode {
    name: String,
    endpoints: Endpoints,
}

impl ServiceNode {
    fn build_endpoint_id(method: &String, path: &String) -> String {
        format!("{}:{}", method, path)
    }

    pub fn from(log_entry: &NormalizedLogEntry) -> Self {
        let endpoint_id = Self::build_endpoint_id(&log_entry.method, &log_entry.path);
        Self {
            name: log_entry.service_name.clone(),
            endpoints: HashMap::from_iter([
                (
                    endpoint_id,
                    EndpointNode::new(
                        log_entry.method.clone(),
                        log_entry.path.clone(),
                        log_entry.trace_id.clone(),
                        log_entry.span_id.clone()
                    ),
                ),
            ]),
        }
    }

    pub fn add_endpoint_from(&mut self, log_entry: &NormalizedLogEntry) {
        let endpoint_id = Self::build_endpoint_id(&log_entry.method, &log_entry.path);

        self.endpoints
            .entry(endpoint_id)
            .and_modify(|endpoint|
                endpoint.add_metadata(log_entry.trace_id.clone(), log_entry.span_id.clone())
            )
            .or_insert(
                EndpointNode::new(
                    log_entry.method.clone(),
                    log_entry.path.clone(),
                    log_entry.trace_id.clone(),
                    log_entry.span_id.clone()
                )
            );
    }
}

struct RequestNode {
    url: String,
    origin: ServiceNode,
    target: (ServiceNode, EndpointNode),
}

pub async fn ingest() -> Result<()> {
    info!("Initiating gcp-log-ingestor");

    let env = load_env();

    info!("Connecting to graph database");
    let service_graph = ServiceGraphBuilder::new(env.database_url, env.graph_name, env.pool).await?;

    info!("Building logging service");
    let gcp_log_ingestor = GCPLogIngestor::new(
        env.project_id,
        env.page_size,
        env.max_pages,
        env.log_filter,
        env.custom_path_regex
    ).await?;

    let trace_ids: Mutex<HashSet<String>> = Mutex::new(HashSet::default());
    let service_nodes: Mutex<HashMap<String, ServiceNode>> = Mutex::new(HashMap::default());

    info!("Running GCP log ingestor");
    gcp_log_ingestor.run(|log_entry| {
        debug!("{:?}", log_entry);
        trace_ids.lock().unwrap().insert(log_entry.trace_id.clone());
        service_nodes
            .lock()
            .unwrap()
            .entry(log_entry.service_name.clone())
            .and_modify(|service_node| service_node.add_endpoint_from(&log_entry))
            .or_insert(ServiceNode::from(&log_entry));
    }).await?;

    info!("{:?}", service_nodes.lock().unwrap());

    Ok(())
}
