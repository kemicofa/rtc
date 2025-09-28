use std::{ collections::{ HashMap, HashSet }, sync::Arc };

use async_trait::async_trait;
use common::{ marc, types::MArc };
use google_cloud_logging_v2::{ client::LoggingServiceV2 };
use anyhow::{ Ok, Result, bail, Error };
use google_cloud_wkt::Timestamp;
use logs_to_graph::{ service_logs::ServiceLogs, service_node_graph::{ self, ServiceNodeGraph } };
use regex::Regex;
use tokio::{ sync::{ Semaphore, mpsc::{ Sender, channel } }, time::{ Duration, sleep } };
use tracing::{ debug, error, info, warn };

use crate::{
    consts::DEFAULT_LOG_FILTER,
    gcp_processor::GcpProcessor,
    normalize::{ get_default_path_normalize_regexes },
    trace::TracesAPI,
};

pub struct GCPServiceLogs {
    project_id: String,
    log_filter: String,
    client: Arc<LoggingServiceV2>,
    page_size: i32,
    path_normalize_regexes: Vec<(String, Vec<Regex>)>,
    traces_api: Arc<TracesAPI>,
}

impl GCPServiceLogs {
    pub async fn new(
        project_id: String,
        page_size: i32,
        log_filter: Option<String>,
        custom_path_normalize_patterns: Vec<String>
    ) -> Result<Self> {
        if project_id.is_empty() {
            bail!("Expected a non empty project_id");
        }

        let mut path_normalize_regexes: Vec<
            (String, Vec<Regex>)
        > = get_default_path_normalize_regexes();

        if custom_path_normalize_patterns.len() > 0 {
            let custom_path_regexes = custom_path_normalize_patterns
                .iter()
                .map(|pattern| Regex::new(pattern).unwrap())
                .collect();

            path_normalize_regexes.push(("custom_id".into(), custom_path_regexes));
        }

        let client = LoggingServiceV2::builder().build().await?; // Uses ADC by default
        let traces_api = TracesAPI::new().await?;

        let now = Timestamp::try_from(std::time::SystemTime::now())?;
        let week_ago = Timestamp::clamp(now.seconds() - 7 * 24 * 60 * 60, 0);

        let mut internal_log_filters: Vec<String> = vec![
            DEFAULT_LOG_FILTER.to_string(),
            format!("timestamp>=\"{}\"", String::from(week_ago))
        ];

        if log_filter.is_some() {
            internal_log_filters.push(log_filter.unwrap());
        }

        let internal_log_filter = internal_log_filters.join(" AND ");

        Ok(Self {
            client: Arc::new(client),
            project_id,
            log_filter: internal_log_filter,
            page_size,
            path_normalize_regexes,
            traces_api: Arc::new(traces_api),
        })
    }
}

const CHANNEL_BUFFER_SIZE: usize = 32;

#[async_trait]
impl ServiceLogs for GCPServiceLogs {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()> {
        let (tx, mut rx) = channel::<Vec<String>>(CHANNEL_BUFFER_SIZE);
        let sender2 = sender.clone();

        let client = Arc::clone(&self.client);
        let traces_api = Arc::clone(&self.traces_api);

        let gcp_processor_marc = marc!(GcpProcessor {
            service_node_graph: ServiceNodeGraph::default(),
            path_normalize_regexes: self.path_normalize_regexes.clone(),
            trace_map: HashMap::default(),
        });

        // To be used in the second thread.
        let gcp_processor_marc_2 = Arc::clone(&gcp_processor_marc);

        let page_size = self.page_size;
        let log_filter = self.log_filter.clone();
        let project_id = self.project_id.clone();

        let t1 = tokio::spawn(async move {
            let mut page_token = String::new();
            let project = format!("projects/{}", project_id);
            loop {
                let result = client
                    .list_log_entries()
                    .set_resource_names([project.clone()])
                    .set_filter(&log_filter)
                    .set_page_size(page_size)
                    .set_order_by("timestamp asc")
                    .set_page_token(&page_token)
                    .send().await;

                if result.is_err() {
                    let e = result.unwrap_err();
                    error!("Failed to fetch log entries: {}", e);
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }

                let response = result.unwrap();

                page_token = response.next_page_token.clone();

                debug!("Found {} results", response.entries.len());

                let mut gcp_processor = gcp_processor_marc.lock().await;
                let mut new_traces: Vec<String> = vec![];
                for log_entry in response.entries {
                    new_traces.push(log_entry.trace.clone());
                    gcp_processor.process_log(log_entry);
                }

                let send_res = sender.send(gcp_processor.service_node_graph.clone()).await;

                if send_res.is_err() {
                    error!(
                        "Failed to send service node graph from the Service Logs to the Service Graph."
                    );
                }

                if new_traces.len() > 0 {
                    let res = tx.send(new_traces).await;

                    // TODO: Figure out what to do if this fails.
                    if res.is_err() {
                        error!("Failed to send message between threads.");
                    }
                }

                if page_token.is_empty() {
                    info!("Next page token was empty");
                    break;
                }

                sleep(Duration::from_secs(30)).await;
            }
        });

        let t2 = tokio::spawn(async move {
            let mut traces_seen: Vec<String> = vec![];

            while let Some(new_traces) = rx.recv().await {
                let mut iter = new_traces
                    .iter()
                    .filter(|new_trace| !traces_seen.contains(&new_trace));

                // Ensures that not more than a 100 are concurrently being processed at once.
                let sem = Arc::new(Semaphore::new(10));
                let mut handles = Vec::new();

                while let Some(new_trace) = iter.next() {
                    let trace = new_trace.to_string();
                    let tmp_traces_api = Arc::clone(&traces_api);

                    let tmp_gcp_processor_marc = Arc::clone(&gcp_processor_marc_2);
                    let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
                    handles.push(
                        tokio::spawn(async move {
                            debug!("Fetching trace information: {}", trace);

                            let res = tmp_traces_api.get_trace(&trace).await;

                            if res.is_err() {
                                let message = format!("Failed to get trace: {}", res.unwrap_err());
                                warn!("{}", message);
                                return trace;
                            }

                            let mut gcp_processor = tmp_gcp_processor_marc.lock().await;

                            gcp_processor.process_trace(res.unwrap());

                            drop(permit);
                            trace
                        })
                    );
                }

                for h in handles {
                    match h.await {
                        Result::Ok(trace) => {
                            traces_seen.push(trace);
                        }
                        Err(_) => {}
                    }
                }

                let send_res = sender2.send(
                    gcp_processor_marc_2.lock().await.service_node_graph.clone()
                ).await;

                if send_res.is_err() {
                    error!(
                        "Failed to send service node graph from the Service Logs to the Service Graph."
                    );
                }
            }
        });

        let (r1, r2) = tokio::join!(t1, t2);

        r1?;
        r2?;

        Ok(())
    }
}
