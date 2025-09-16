use std::{ collections::{ HashMap, HashSet } };

use async_trait::async_trait;
use google_cloud_logging_v2::{ client::LoggingServiceV2 };
use anyhow::{ Ok, Result, bail };
use google_cloud_wkt::Timestamp;
use logs_to_graph::{
    service_logs::ServiceLogs,
    service_node_graph::{ Operation, ServiceName, ServiceNodeGraph },
};
use tokio::{ sync::{ mpsc::{ Sender } }, time::{ Duration, sleep } };
use tracing::{ debug, error, info, warn };

use crate::{
    consts::DEFAULT_LOG_FILTER,
    normalize::{ normalize_log_entry },
    trace::TracesAPI,
    types::{ SpanId, TraceId },
};

pub struct GCPServiceLogs {
    project_id: String,
    log_filter: String,
    client: LoggingServiceV2,
    page_size: i32,
    max_pages: i32,
    custom_path_regex: Option<String>,
    traces_api: TracesAPI,
}

impl GCPServiceLogs {
    pub async fn new(
        project_id: String,
        page_size: i32,
        max_pages: i32,
        log_filter: Option<String>,
        custom_path_regex: Option<String>
    ) -> Result<Self> {
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
            client,
            project_id,
            log_filter: internal_log_filter,
            page_size,
            max_pages,
            custom_path_regex,
            traces_api,
        })
    }
}

#[async_trait]
impl ServiceLogs for GCPServiceLogs {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()> {
        let mut service_node_graph = ServiceNodeGraph::default();
        let mut handled_trace_ids: HashSet<String> = HashSet::new();
        let mut trace_id_to_span_id_to_service_info: HashMap<
            TraceId,
            HashMap<SpanId, (ServiceName, Operation)>
        > = HashMap::new();

        let mut page_token = String::new();

        let mut page: i32 = 0;
        loop {
            page += 1;

            if page > self.max_pages {
                break;
            }

            info!("{}/{}. Fetching {} log entries", page, self.max_pages, self.page_size);

            // TODO: check how next page tokens are handled in this library.
            // Might be possible that this will resend the same page over and over again.
            // If so then will need to move this section (up to the send()) before the loop.
            let result = self.client
                .list_log_entries()
                .set_resource_names([format!("projects/{}", self.project_id)])
                .set_filter(&self.log_filter)
                .set_page_size(self.page_size)
                .set_order_by("timestamp asc")
                .set_page_token(&page_token)
                .send().await;

            if result.is_err() {
                // TODO: probably shouldn't bail here and just log a warning
                // sleep for 10 seconds and hope and pray the connection works
                // next round.
                bail!("Failed to fetch log entries: {}", result.unwrap_err());
            }

            let response = result.unwrap();

            page_token = response.next_page_token.clone();

            debug!("Found {} results on page {}", response.entries.len(), page);

            let mut normalized_log_entry_unique_errors: HashMap<String, usize> = Default::default();

            for e in response.entries {
                let normalized_log_entry_result = normalize_log_entry(
                    e,
                    self.custom_path_regex.clone()
                );

                if normalized_log_entry_result.is_err() {
                    let e = normalized_log_entry_result.unwrap_err();
                    normalized_log_entry_unique_errors
                        .entry(e.to_string())
                        .and_modify(|count| {
                            *count += 1;
                        })
                        .or_insert(1);
                    continue;
                }

                let normalized_log_entry = normalized_log_entry_result.unwrap();

                // For now only HTTP is supported, however once other types of
                // protocols are added, need to update NormalizedLogEntry
                // to handle other cases.
                let operation = Operation::Http(
                    normalized_log_entry.http_request.0,
                    normalized_log_entry.http_request.1
                );

                // Will create the service if it does not exist.
                service_node_graph.add_operation_to_service(
                    normalized_log_entry.service_name.clone(),
                    operation.clone()
                );

                if normalized_log_entry.trace_id.is_none() {
                    warn!("No trace_id was found on log entry");
                    continue;
                }

                if normalized_log_entry.span_id.is_none() {
                    warn!("No span_id was found on log entry");
                    continue;
                }

                let trace_id = normalized_log_entry.trace_id.unwrap();
                let span_id = normalized_log_entry.span_id.unwrap();

                // TODO: extract this logic in its own struct.
                trace_id_to_span_id_to_service_info
                    .entry(trace_id.clone())
                    .and_modify(|span_ids_to_service_info_map| {
                        span_ids_to_service_info_map.insert(span_id.clone(), (
                            normalized_log_entry.service_name.clone(),
                            operation.clone(),
                        ));
                    })
                    .or_insert(
                        HashMap::from_iter([
                            (
                                span_id.clone(),
                                (normalized_log_entry.service_name, operation.clone()),
                            ),
                        ])
                    );
            }

            if normalized_log_entry_unique_errors.len() > 0 {
                warn!("Failed to normalize log entry: {:?}", normalized_log_entry_unique_errors);
            }

            // Creating associations between Service Nodes. This is where the magic happens.
            for (
                trace_id,
                span_ids_to_service_info_map,
            ) in trace_id_to_span_id_to_service_info.iter() {
                if handled_trace_ids.contains(trace_id) {
                    continue;
                }
                // Note: loading a trace takes quite a while.
                let trace_result = self.traces_api.get_trace(&self.project_id, &trace_id).await;

                if trace_result.is_err() {
                    let e = trace_result.unwrap_err();
                    warn!("Failed to load trace: {}", e);
                    continue;
                }

                let trace = trace_result.unwrap();

                for span in trace.spans.iter() {
                    // If the span does not have a parent_span_id
                    // then it means that it is the first of the trace
                    // and there is nothing to link it to
                    if span.parent_span_id.is_none() {
                        continue;
                    }

                    let parent_span_id = span.parent_span_id.clone().unwrap();

                    // if for whatever reason we haven't previously registered
                    // the span_id or the parent_span_id to a service, we won't be able
                    // to make an association. This isn't a big deal as these associations
                    // can be made later with other traces with a similar hierarchy.
                    // Note: if there is no match, it could be caused because some log_entries
                    // with the associated span_ids have not yet been processed (they could be on a different page).
                    // Which is important to keep the page_sizes higher to avoid such issues.
                    //
                    // It could be feasible one day to keep in memory the traces until
                    // we have a complete map.
                    if
                        !span_ids_to_service_info_map.contains_key(&span.span_id) ||
                        !span_ids_to_service_info_map.contains_key(&parent_span_id)
                    {
                        continue;
                    }

                    let invoker_service_info = span_ids_to_service_info_map
                        .get(&parent_span_id)
                        .unwrap()
                        .clone();
                    let target_service_info = span_ids_to_service_info_map
                        .get(&span.span_id)
                        .unwrap()
                        .clone();

                    service_node_graph.add_target_to_service(
                        invoker_service_info.0,
                        target_service_info.0,
                        target_service_info.1
                    );

                    // TODO: once we've successfully processed a trace
                    // we should remove it from trace_id_to_span_id_to_service_info.
                    // Since technically it can infinitely increase.
                }

                handled_trace_ids.insert(trace_id.clone());
            }

            // Once we've processed a page, then send the service_node_graph to be handled by the graph db.
            let send_res = sender.send(service_node_graph.clone()).await;

            if send_res.is_err() {
                let e = send_res.unwrap_err();
                warn!("Failed to send the updated ServiceNodeGraph. Will try again next iteration. {}", e);
            }

            if page_token.is_empty() {
                debug!("Stopping because next page token is empty");
                break;
            }

            sleep(Duration::from_secs(2)).await; // basic polling interval
        }

        Ok(())
    }
}
