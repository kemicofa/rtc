pub fn build_trace_path(project_id: String, trace_id: String) -> String {
    format!("projects/{}/traces/{}", project_id, trace_id)
}
