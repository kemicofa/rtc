use std::collections::{ HashMap, HashSet };

use google_cloud_logging_v2::model::LogEntry;
use logs_to_graph::service_node_graph::{ Operation, ServiceNodeGraph };
use tracing::{ error, warn };
use crate::trace::Trace;
use crate::types::{ Trace as TraceType, SpanId };
use crate::normalize::{ normalize_path };
use crate::utils::build_trace_path;
use regex::Regex;
use logs_to_graph::service_node_graph::ServiceName;

pub struct GcpProcessor {
    pub service_node_graph: ServiceNodeGraph,
    pub path_normalize_regexes: Vec<(String, Vec<Regex>)>,
    pub trace_map: HashMap<TraceType, HashMap<SpanId, (ServiceName, Operation)>>,
}

impl GcpProcessor {
    /**
     * Logs that have already been validated to be a cloud_run_revision.
     */
    fn process_cloud_run_revision_log_entry(&mut self, log_entry: LogEntry) {
        // For now, only processing logs that have a http_request object
        // Future, should support other protocols.
        if log_entry.http_request.is_none() {
            return;
        }

        let service_name_opt = log_entry.resource
            .map(|resource| resource.labels.get("service_name").cloned())
            .unwrap_or(None);

        if service_name_opt.is_none() {
            warn!("Detected a cloud run revision log that did not have a service name set");
            return;
        }

        let http_request = log_entry.http_request.unwrap();
        let path_result = normalize_path(
            http_request.request_url.as_str(),
            self.path_normalize_regexes.clone()
        );

        // If there was a malformed URL, we simply stop and move on.
        if path_result.is_err() {
            error!(
                "Unable to process GCP log entry with malformed URL: {}",
                path_result.unwrap_err()
            );
            return;
        }

        let service_name = service_name_opt.unwrap();

        let operation = Operation::Http {
            method: http_request.request_method,
            path: path_result.unwrap(),
        };

        self.service_node_graph.add_operation_to_service(service_name.clone(), operation.clone());

        // --- This section associates traces to the service name

        let span_id = log_entry.span_id;

        self.trace_map
            .entry(log_entry.trace)
            .and_modify(|span_to_service_operation| {
                span_to_service_operation.insert(span_id.clone(), (
                    service_name.clone(),
                    operation.clone(),
                ));
            })
            .or_insert(
                HashMap::from_iter([(span_id.clone(), (service_name.clone(), operation.clone()))])
            );
    }

    pub fn process_log(&mut self, log_entry: LogEntry) {
        if let Some(resource) = log_entry.resource.as_ref() {
            match resource.r#type.as_str() {
                "cloud_run_revision" => self.process_cloud_run_revision_log_entry(log_entry),
                _ => {}
            }
        }
    }

    pub fn process_trace(&mut self, trace: Trace) {
        let trace_string = build_trace_path(trace.project_id, trace.trace_id);
        let trace_entry = self.trace_map.get(&trace_string);

        if trace_entry.is_none() {
            warn!("Attempted to process a trace linked to nothing.");
            return;
        }

        let span_ids_to_service_info_map = trace_entry.unwrap();

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

            self.service_node_graph.add_target_to_service(
                invoker_service_info.0,
                target_service_info.0,
                target_service_info.1
            );

            // TODO: once we've successfully processed a trace
            // we should remove it from trace_id_to_span_id_to_service_info.
            // Since technically it can infinitely increase.
        }
    }
}
