use anyhow::{ ensure, bail, Result, Ok };
use google_cloud_logging_v2::model::LogEntry;
use logs_to_graph::service_node_graph::{ HttpMethod, HttpPath, ServiceName };
use url::Url;
use regex::Regex;

use crate::types::{ SpanId, TraceId };

#[derive(Debug, Clone)]
pub enum ResourceType {
    CloudRunRevision,
}

#[derive(Debug, Clone)]
pub struct NormalizedLogEntry {
    pub path: HttpPath,
    pub span_id: SpanId,
    pub resource_type: ResourceType,
    pub service_name: ServiceName,
    pub trace_id: TraceId,
    pub method: HttpMethod,
}

/// Normalizes a path by replacing id's and uuid's
fn normalize_path(url_str: &str, custom_path_regex: Option<String>) -> Result<String> {
    let url = Url::parse(url_str)?;
    let path_segments: Vec<&str> = match url.path_segments() {
        Some(segments) => segments.collect(),
        None => bail!("Cannot extract segments from URL path"),
    };

    let mut regexes: Vec<(&str, Regex)> = vec![
        ("id", Regex::new(r"^\d+$")?),
        (
            "uuid",
            Regex::new(
                r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
            )?,
        )
    ];

    if custom_path_regex.is_some() {
        regexes.push(("custom_id", Regex::new(custom_path_regex.unwrap().as_str())?));
    }

    let mut normalized_segments = Vec::new();
    let mut segment_iter = path_segments.iter().peekable();

    while let Some(segment) = segment_iter.next() {
        let mut matched = false;
        for (r#type, regex) in regexes.clone() {
            if !regex.is_match(&segment) {
                continue;
            }

            if let Some(prev_segment) = normalized_segments.last() {
                normalized_segments.push(format!("{{{}_{}}}", prev_segment, r#type));
            } else {
                normalized_segments.push(format!("{{{}}}", r#type).to_string());
            }

            matched = true;
        }

        if !matched {
            normalized_segments.push(segment.to_string());
        }
    }

    Ok(format!("/{}", normalized_segments.join("/")))
}

pub fn normalize_log_entry(
    e: LogEntry,
    custom_path_regex: Option<String>
) -> Result<NormalizedLogEntry> {
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

    let service_name: String = match resource.labels.get("service_name") {
        Some(name) => name.into(),
        None => bail!("Resource of type 'cloud_run_revision' did not have a service_name"),
    };

    let path = normalize_path(&request_url, custom_path_regex)?;

    let trace_id: String = match e.trace.split("/").last() {
        Some(id) => id.into(),
        None => bail!("Unable to extract trace_id from trace"),
    };

    Ok(NormalizedLogEntry {
        path,
        service_name,
        span_id,
        trace_id,
        resource_type: ResourceType::CloudRunRevision,
        method: http_request.request_method.to_uppercase(),
    })
}

#[cfg(test)]
mod test {
    use crate::normalize::normalize_path;

    #[test]
    fn should_replace_ids() {
        let url = "https://test.com/users/12345/books/12345";
        let expect = "/users/{users_id}/books/{books_id}".to_string();
        let res = normalize_path(url, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_consecutive_ids() {
        let url = "https://test.com/users/12345/12345/books/12345/12345";
        let expect =
            "/users/{users_id}/{{users_id}_id}/books/{books_id}/{{books_id}_id}".to_string();
        let res = normalize_path(url, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_uuids() {
        let url = "https://test.com/users/91366bf0-4c97-4832-af68-452c51ca38eb/books/12345";
        let expect = "/users/{users_uuid}/books/{books_id}".to_string();
        let res = normalize_path(url, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_using_custom_path_regex() {
        let url =
            "https://test.com/users/91366bf0-4c97-4832-af68-452c51ca38eb/books/12345/car/prefix-12345";
        let expect = "/users/{users_uuid}/books/{books_id}/car/{car_custom_id}".to_string();
        let res = normalize_path(url, Some("^prefix-\\d+$".into()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }
}
