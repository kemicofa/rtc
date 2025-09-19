use std::time::Duration;

use anyhow::{ Ok, Result, bail };
use async_trait::async_trait;
use datadog_api_client::{
    datadog::{ APIKey, Configuration },
    datadogV1::{
        api_authentication::AuthenticationAPI,
        api_logs::LogsAPI,
        model::{ LogsListRequest, LogsListRequestTime, LogsSort },
    },
};
use logs_to_graph::{ service_logs::ServiceLogs, service_node_graph::ServiceNodeGraph };
use serde_json::to_string_pretty;
use tokio::sync::mpsc::Sender;
use tracing::{ debug, error, warn };

pub struct DatadogServiceLog {
    logs_api: LogsAPI,
}

impl DatadogServiceLog {
    pub async fn new(
        site: String,
        api_key: Option<String>,
        app_key: Option<String>
    ) -> Result<Self> {
        let mut cfg = Configuration::new();

        cfg.server_index = 0;
        cfg.server_variables.insert("site".into(), site);

        if let Some(api_key) = api_key {
            warn!("Recommended to pass the api key via the DD_API_KEY env");
            cfg.set_auth_key("apiKeyAuth", APIKey {
                key: api_key,
                prefix: "".to_owned(),
            });
        }

        if let Some(app_key) = app_key {
            warn!("Recommended to pass the app key via the DD_APP_KEY env");
            cfg.set_auth_key("appKeyAuth", APIKey {
                key: app_key,
                prefix: "".to_owned(),
            });
        }

        let api = AuthenticationAPI::with_config(cfg.clone());

        let resp = api.validate().await?;

        debug!("Authenticated with Datadog: {:?}", resp);

        let logs_api = LogsAPI::with_config(cfg);

        Ok(Self {
            logs_api,
        })
    }
}

#[async_trait]
impl ServiceLogs for DatadogServiceLog {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()> {
        let from_one_week_ago = chrono::Utc::now() - chrono::Duration::weeks(1);
        let to_now = chrono::Utc::now();
        let mut next_page_token: Option<String> = None;

        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let mut req = LogsListRequest::new(LogsListRequestTime::new(from_one_week_ago, to_now))
                .limit(100)
                .sort(LogsSort::TIME_ASCENDING);

            if next_page_token.is_some() {
                req = req.start_at(next_page_token.clone().unwrap());
            }

            let res = self.logs_api.list_logs(req).await;

            if res.is_err() {
                bail!("[Datadog][list_logs] Failed fetching logs: {}", res.unwrap_err());
            }

            let logs_list_response = res.unwrap();

            // Why do they wrap the next log id in a double option. FML.
            next_page_token = logs_list_response.next_log_id.unwrap_or(None);

            if logs_list_response.logs.is_none() {
                debug!("[Datadog][list_logs] No logs were found for page");
                continue;
            }

            let logs = logs_list_response.logs.unwrap();

            for log in logs {
                // let service_name = log.content.map(|content| { content.service }).unwrap_or(None);
                debug!("{}", to_string_pretty(&log).unwrap());
            }

            if next_page_token.is_none() {
                break;
            }
        }

        Ok(())
    }
}
