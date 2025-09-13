use google_cloud_logging_v2::{ client::LoggingServiceV2 };
use anyhow::{ Ok, Result };
use tokio::time::{ sleep, Duration };
use tracing::{ debug, info, warn };

use crate::{ consts::DEFAULT_LOG_FILTER, normalize::{ NormalizedLogEntry, normalize_log_entry } };

mod consts;
pub mod normalize;

pub struct GCPLogIngestor {
    project_id: String,
    log_filter: String,
    client: LoggingServiceV2,
    page_size: i32,
    max_pages: i32,
    custom_path_regex: Option<String>,
}

impl GCPLogIngestor {
    pub async fn new(
        project_id: String,
        page_size: i32,
        max_pages: i32,
        log_filter: Option<String>,
        custom_path_regex: Option<String>
    ) -> Result<Self> {
        let client = LoggingServiceV2::builder().build().await?; // Uses ADC by default

        let internal_log_filter: String = log_filter.map_or(DEFAULT_LOG_FILTER.into(), |v|
            [DEFAULT_LOG_FILTER, v.as_str()].join(" AND ")
        );

        Ok(Self {
            client,
            project_id,
            log_filter: internal_log_filter,
            page_size,
            max_pages,
            custom_path_regex,
        })
    }

    pub async fn run<F>(&self, callback: F) -> Result<()> where F: Fn(NormalizedLogEntry) -> () {
        let mut page: i32 = 0;
        loop {
            page += 1;

            if page > self.max_pages {
                break;
            }

            info!("{}/{}. Fetching {} log entries", page, self.max_pages, self.page_size);
            let response = self.client
                .list_log_entries()
                .set_resource_names([format!("projects/{}", self.project_id)])
                .set_filter(&self.log_filter)
                .set_page_size(self.page_size)
                .send().await?;

            debug!("Found {} results on page {}", response.entries.len(), page);

            for e in response.entries {
                debug!("{:?}", e);
                let log_entry = normalize_log_entry(e, self.custom_path_regex.clone());

                if log_entry.is_err() {
                    warn!("Skipping log entry: {}", log_entry.unwrap_err());
                    continue;
                }

                callback(log_entry.unwrap());
            }

            if response.next_page_token.is_empty() {
                debug!("Stopping because next page token is empty");
                break;
            }

            sleep(Duration::from_secs(2)).await; // basic polling interval
        }

        Ok(())
    }
}
