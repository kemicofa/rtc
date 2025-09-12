use anyhow::{ ensure, bail, Result, Ok };
use google_cloud_logging_v2::model::LogEntry;
use url::Url;

#[derive(Debug, Clone)]
pub enum ResourceType {
    CloudRunRevision,
}

#[derive(Debug, Clone)]
pub struct NormalizedLogEntry {
    pub path: String,
    pub span_id: String,
    pub resource_type: ResourceType,
    pub service_name: String,
    pub trace_id: String,
    pub method: String,
}

fn extract_path(input: &str) -> Option<String> {
    let url = Url::parse(input);

    if url.is_err() {
        return None;
    }

    Some(url.unwrap().path().to_string())
}

pub fn normalize_log_entry(e: LogEntry) -> Result<NormalizedLogEntry> {
    let span_id = e.span_id;
    ensure!(!span_id.is_empty(), "span_id is empty");
    ensure!(e.resource.is_some(), "missing resource");
    ensure!(e.http_request.is_some(), "missing http_request");

    let resource = e.resource.unwrap();
    let http_request = e.http_request.unwrap();

    let request_url = http_request.request_url;

    if resource.r#type.as_str() != "cloud_run_revision" {
        bail!("Resource must be of type 'cloud_run_revision'");
    }

    let service_name = resource.labels.get("service_name");

    if service_name.is_none() {
        bail!("Resource of type 'cloud_run_revision' did not have a service_name");
    }

    let path = extract_path(&request_url);

    if path.is_none() {
        bail!("Unable to extract path from request_url");
    }

    let trace_id = e.trace.split("/").last();

    if trace_id.is_none() {
        bail!("Unable to extract trace_id from trace");
    }

    Ok(NormalizedLogEntry {
        path: path.unwrap(),
        service_name: service_name.unwrap().into(),
        span_id,
        trace_id: trace_id.unwrap().into(),
        resource_type: ResourceType::CloudRunRevision,
        method: http_request.request_method,
    })
}
