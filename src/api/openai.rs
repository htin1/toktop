use crate::models::{OpenAIBucket, OpenAICostResponse, OpenAICostResult};
use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde_json;

#[derive(Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1/organization".to_string(),
        }
    }

    pub async fn fetch_costs(&self) -> Result<Vec<OpenAIBucket<OpenAICostResult>>> {
        let start_time = Utc::now() - chrono::Duration::days(7);
        let start_ts = start_time.timestamp();
        let mut all_buckets = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = format!(
                "{}/costs?start_time={}&group_by=line_item&limit=7",
                self.base_url, start_ts
            );
            if let Some(ref p) = page {
                url = format!("{}&page={}", url, p);
            }
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await
                .context("Failed to fetch costs")?;
            let status = response.status();
            let text = response.text().await.context("Failed to read response")?;
            if !status.is_success() {
                return Err(anyhow::anyhow!("API error: {} - {}", status, text));
            }

            let resp: OpenAICostResponse = serde_json::from_str(&text).context(format!(
                "Parse error. Response: {}",
                text.chars().take(200).collect::<String>()
            ))?;
            all_buckets.extend(resp.data);
            if !resp.has_more {
                break;
            }
            page = resp.next_page;
        }
        Ok(all_buckets)
    }
}
