// Note: wildcards are not allowed in the log filter for whatever reason.
pub const DEFAULT_LOG_FILTER: &str = "resource.type:cloud_run_revision AND http_request:*";

pub const PATH_NORMALIZE_PATTERNS: [(&str, &str); 2] = [
    ("id", r"^\d+$"),
    (
        "uuid",
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
    ),
];
