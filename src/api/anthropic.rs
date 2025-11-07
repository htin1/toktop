use crate::models::{
    AnthropicCostBucket, AnthropicCostResponse, AnthropicUsageResponse, AnthropicUsageTimeBucket,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json;

#[derive(Clone)]
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1/organizations".to_string(),
        }
    }

    pub async fn fetch_costs(
        &self,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<AnthropicCostBucket>> {
        let start = start_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let base_url = format!("{}/cost_report", self.base_url);
        let mut all_data = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut req = self
                .client
                .get(&base_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .query(&[
                    ("starting_at", start.as_str()),
                    ("group_by[]", "description"),
                ]);
            if let Some(ref p) = page {
                req = req.query(&[("page", p.as_str())]);
            }
            let response = req.send().await.context("Failed to send request")?;
            let status = response.status();
            let text = response.text().await.context("Failed to read response body")?;
            if !status.is_success() {
                return Err(anyhow::anyhow!("API error: {} - {}", status, text));
            }
            let resp: AnthropicCostResponse = serde_json::from_str(&text).context(format!(
                "Failed to parse response. Status: {}. Response: {}",
                status,
                text.chars().take(500).collect::<String>()
            ))?;
            all_data.extend(resp.data);
            if !resp.has_more {
                break;
            }
            page = resp.next_page;
        }
        Ok(all_data)
    }

    pub async fn fetch_usage(
        &self,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<AnthropicUsageTimeBucket>> {
        let start = start_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let base_url = format!("{}/usage_report/messages", self.base_url);
        let mut all_data = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut req = self
                .client
                .get(&base_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .query(&[
                    ("starting_at", start.as_str()),
                    ("group_by[]", "model"),
                    ("bucket_width", "1d"),
                ]);
            if let Some(ref p) = page {
                req = req.query(&[("page", p.as_str())]);
            }
            let response = req.send().await.context("Failed to send request")?;
            let status = response.status();
            let text = response.text().await.context("Failed to read response body")?;
            if !status.is_success() {
                return Err(anyhow::anyhow!("API error: {} - {}", status, text));
            }
            let resp: AnthropicUsageResponse = serde_json::from_str(&text).context(format!(
                "Failed to parse response. Status: {}. Response: {}",
                status,
                text.chars().take(500).collect::<String>()
            ))?;
            all_data.extend(resp.data);
            if !resp.has_more {
                break;
            }
            page = resp.next_page;
        }
        Ok(all_data)
    }
}
