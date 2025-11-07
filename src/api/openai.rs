use crate::models::{OpenAIBucket, OpenAICostResponse, OpenAICostResult, OpenAIUsageResponse};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
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

    async fn fetch_usage_endpoint(
        &self,
        endpoint: &str,
        start_ts: i64,
    ) -> Result<Vec<crate::models::OpenAIUsageBucket>> {
        let mut all_buckets = Vec::new();
        let mut page: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/usage/{}?start_time={}&interval=1d&group_by=model&group_by=api_key_id",
                self.base_url, endpoint, start_ts
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
                .context(format!("Failed to fetch {} usage", endpoint))?;

            let status = response.status();
            let text = response.text().await.context("Failed to read response")?;

            if !status.is_success() {
                return Err(anyhow::anyhow!(
                    "API error for {}: {} - {}",
                    endpoint,
                    status,
                    text
                ));
            }

            let resp: OpenAIUsageResponse = serde_json::from_str(&text).context(format!(
                "Failed to parse {} usage response: {}",
                endpoint,
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

    pub async fn fetch_usage(
        &self,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<crate::models::OpenAIUsageBucket>> {
        let start_ts = start_time.timestamp();

        // Fetch from all three usage endpoints in parallel
        let (completions_result, embeddings_result, images_result) = tokio::join!(
            self.fetch_usage_endpoint("completions", start_ts),
            self.fetch_usage_endpoint("embeddings", start_ts),
            self.fetch_usage_endpoint("images", start_ts),
        );

        let mut all_buckets = Vec::new();

        // Collect results, ignoring errors for individual endpoints
        if let Ok(mut buckets) = completions_result {
            all_buckets.append(&mut buckets);
        }
        if let Ok(mut buckets) = embeddings_result {
            all_buckets.append(&mut buckets);
        }
        if let Ok(mut buckets) = images_result {
            all_buckets.append(&mut buckets);
        }

        Ok(all_buckets)
    }
}
