// Note: wildcards are not allowed in the log filter for whatever reason.
pub const DEFAULT_LOG_FILTER: &str = "resource.type:cloud_run_revision AND http_request:*";
