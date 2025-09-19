use anyhow::{ Ok, Result, bail };
use async_trait::async_trait;
use datadog_api_client::{
    datadog::{ APIKey, Configuration },
    datadogV1::{
        api_authentication::AuthenticationAPI,
        api_logs::LogsAPI,
        model::{ LogsListRequest, LogsListRequestTime },
    },
};
use logs_to_graph::{ service_logs::ServiceLogs, service_node_graph::ServiceNodeGraph };
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

        let req = LogsListRequest::new(LogsListRequestTime::new(from_one_week_ago, to_now));

        debug!("GETTING LOGS");
        match self.logs_api.list_logs(req).await {
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
