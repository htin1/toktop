use crate::models::{
    OpenAIBucket, OpenAICostResponse, OpenAICostResult, OpenAIProjectApiKey,
    OpenAIProjectApiKeysResponse, OpenAIProjectsResponse, OpenAIUsageResponse,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::collections::{HashMap, HashSet};

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

    pub async fn fetch_costs(
        &self,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<OpenAIBucket<OpenAICostResult>>> {
        let start_ts = start_time.timestamp();
        let mut all_buckets = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = format!(
                "{}/costs?start_time={}&group_by=line_item&limit=30",
                self.base_url, start_ts
            );
            if let Some(ref p) = page {
                url = format!("{}&page={}", url, urlencoding::encode(p));
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
                text.chars().take(500).collect::<String>()
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
                url = format!("{}&page={}", url, urlencoding::encode(p));
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

        let (completions_result, embeddings_result, images_result) = tokio::join!(
            self.fetch_usage_endpoint("completions", start_ts),
            self.fetch_usage_endpoint("embeddings", start_ts),
            self.fetch_usage_endpoint("images", start_ts),
        );

        let mut all_buckets = Vec::new();
        let mut endpoint_errors = Vec::new();

        for (result, name) in [
            (completions_result, "completions"),
            (embeddings_result, "embeddings"),
            (images_result, "images"),
        ] {
            match result {
                Ok(mut buckets) => all_buckets.append(&mut buckets),
                Err(e) => endpoint_errors.push(format!("{}: {}", name, e)),
            }
        }

        if all_buckets.is_empty() && !endpoint_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to fetch usage from any endpoint: {}",
                endpoint_errors.join("; ")
            ));
        }

        Ok(all_buckets)
    }

    pub async fn fetch_projects(&self) -> Result<Vec<crate::models::OpenAIProject>> {
        let mut all_projects = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let mut url = format!("{}/projects", self.base_url);
            if let Some(ref a) = after {
                url = format!("{}?after={}", url, a);
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

    async fn fetch_api_keys_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<OpenAIProjectApiKey>> {
        let mut all_api_keys = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let mut url = format!("{}/projects/{}/api_keys", self.base_url, project_id);
            if let Some(ref a) = after {
                url = format!("{}?after={}", url, a);
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .send()
                .await
                .context(format!(
                    "Failed to fetch API keys for project {}",
                    project_id
                ))?;

            let status = response.status();
            let text = response.text().await.context("Failed to read response")?;

            if !status.is_success() {
                return Err(anyhow::anyhow!("API error: {} - {}", status, text));
            }

            let resp: OpenAIProjectApiKeysResponse =
                serde_json::from_str(&text).context(format!(
                    "Failed to parse API keys response for project {}",
                    project_id
                ))?;

            all_api_keys.extend(resp.data);

            if !resp.has_more {
                break;
            }

            if let Some(id) = resp.last_id {
                after = Some(id);
            } else {
                break;
            }
        }

        Ok(all_api_keys)
    }

    pub async fn fetch_api_key_names_for_ids(
        &self,
        api_key_ids: &[String],
    ) -> Result<HashMap<String, String>> {
        let projects = self
            .fetch_projects()
            .await
            .context("Failed to fetch projects")?;
        let api_key_ids_set: HashSet<&String> = api_key_ids.iter().collect();

        let fetch_tasks: Vec<_> = projects
            .iter()
            .map(|project| {
                let client = self.clone();
                let project_id = project.id.clone();
                tokio::spawn(async move { client.fetch_api_keys_for_project(&project_id).await })
            })
            .collect();

        let mut api_key_map = HashMap::new();
        for task in fetch_tasks {
            if let Ok(Ok(api_keys)) = task.await {
                for api_key in api_keys {
                    if api_key_ids_set.contains(&api_key.id) {
                        api_key_map
                            .insert(api_key.id.clone(), api_key.name.clone().unwrap_or_default());
                    }
                }
            }
        }

        Ok(api_key_map)
    }
}
