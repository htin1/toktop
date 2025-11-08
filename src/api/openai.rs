use crate::models::{
    OpenAIBucket, OpenAICostResponse, OpenAICostResult, OpenAIProjectApiKey, OpenAIProjectsResponse,
    OpenAIUsageResponse,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json;
use std::collections::HashMap;

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

    pub async fn fetch_projects(&self) -> Result<Vec<crate::models::OpenAIProject>> {
        let mut all_projects = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let mut url = format!("{}/projects", self.base_url);
            if let Some(ref a) = after {
                url = format!("{}&after={}", url, a);
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .send()
                .await
                .context("Failed to fetch projects")?;

            let status = response.status();
            let text = response.text().await.context("Failed to read response")?;

            if !status.is_success() {
                return Err(anyhow::anyhow!("API error: {} - {}", status, text));
            }

            let resp: OpenAIProjectsResponse = serde_json::from_str(&text).context(format!(
                "Failed to parse projects response: {}",
                text.chars().take(200).collect::<String>()
            ))?;

            all_projects.extend(resp.data);

            if !resp.has_more {
                break;
            }

            after = resp.last_id;
        }

        Ok(all_projects)
    }

    pub async fn fetch_api_key_by_id(
        &self,
        project_id: &str,
        api_key_id: &str,
    ) -> Result<Option<OpenAIProjectApiKey>> {
        let url = format!(
            "{}/projects/{}/api_keys/{}",
            self.base_url, project_id, api_key_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .context(format!("Failed to fetch API key {} from project {}", api_key_id, project_id))?;

        let status = response.status();
        let text = response.text().await.context("Failed to read response")?;

        if status == 404 {
            return Ok(None);
        }

        if !status.is_success() {
            return Err(anyhow::anyhow!("API error: {} - {}", status, text));
        }

        let api_key: OpenAIProjectApiKey = serde_json::from_str(&text).context(format!(
            "Failed to parse API key response: {}",
            text.chars().take(200).collect::<String>()
        ))?;

        Ok(Some(api_key))
    }

    pub async fn fetch_api_key_names_for_ids(
        &self,
        api_key_ids: &[String],
    ) -> Result<HashMap<String, String>> {
        let projects = self.fetch_projects().await.context("Failed to fetch projects")?;
        let mut api_key_map = HashMap::new();

        for api_key_id in api_key_ids {
            for project in &projects {
                match self
                    .fetch_api_key_by_id(&project.id, api_key_id)
                    .await?
                {
                    Some(api_key) => {
                        api_key_map.insert(api_key.id.clone(), api_key.name.clone());
                        break;
                    }
                    None => continue,
                }
            }
        }

        Ok(api_key_map)
    }
}
