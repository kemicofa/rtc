use anyhow::{ Ok, Result, bail };
use serde::Deserialize;

use crate::creds::load_creds;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub project_id: String,
    pub trace_id: String,
    pub spans: Vec<Span>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Span {
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

pub struct TracesAPI {
    token: String,
}

impl TracesAPI {
    pub async fn new() -> Result<Self> {
        let token = load_creds().await?;

        Ok(Self {
            token,
        })
    }

    pub async fn get_trace(&self, project_id: String, trace_id: String) -> Result<Trace> {
        let url = format!(
            "https://cloudtrace.googleapis.com/v1/projects/{}/traces/{}",
            project_id,
            trace_id
        );

        let bearer = format!("Bearer {}", self.token);
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header(reqwest::header::AUTHORIZATION, bearer)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            bail!("Trace API error: {} — {}", status, body);
        }

        // If you want typed structs, define them; for brevity we just pretty-print the JSON.
        let trace: Trace = resp.json::<Trace>().await?;

        Ok(trace)
    }
}
