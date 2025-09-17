use anyhow::{ Ok, Result, bail };
use async_trait::async_trait;
use datadog_api_client::{
    datadog::{ APIKey, Configuration },
    datadogV1::{ api_logs::LogsAPI, model::{ LogsListRequest, LogsListRequestTime } },
};
use logs_to_graph::{ service_logs::ServiceLogs, service_node_graph::ServiceNodeGraph };
use tokio::sync::mpsc::Sender;
use tracing::debug;

pub struct DatadogServiceLog {
    cfg: Configuration,
}

impl DatadogServiceLog {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let mut cfg = Configuration::new();

        // if let Some(api_key) = api_key {
        //     cfg.set_auth_key("apiKeyAuth", APIKey {
        //         key: api_key,
        //         prefix: "".to_owned(),
        //     });
        // }

        Ok(Self {
            cfg,
        })
    }
}

#[async_trait]
impl ServiceLogs for DatadogServiceLog {
    async fn run(&self, sender: Sender<ServiceNodeGraph>) -> Result<()> {
        let from_one_week_ago = chrono::Utc::now() - chrono::Duration::weeks(1);
        let to_now = chrono::Utc::now();

        let req = LogsListRequest::new(LogsListRequestTime::new(from_one_week_ago, to_now));
        let api = LogsAPI::with_config(self.cfg.clone());

        debug!("GETTING LOGS");
        match api.list_logs(req).await {
            Result::Ok(response) => {
                if let Some(data) = response.logs {
                    debug!("LOGS FOUND BABY: {:?}", data);
                } else {
                    debug!("No longs found rip");
                }
            }
            Err(e) => {
                debug!("{}", e);
                bail!("Didn't work {}", e);
            }
        }
        debug!("DONE GETTING LOGS");
        Ok(())
    }
}
