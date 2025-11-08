use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AnthropicApiKeyResponse {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub status: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub created_at: Option<String>,
}

#[derive(Clone)]
pub struct DailyData {
    pub date: DateTime<Utc>,
    pub cost: f64,
    pub line_item: Option<String>,
}

#[derive(Clone)]
pub struct DailyUsageData {
    pub date: DateTime<Utc>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_key_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone)]
pub struct UsageData {
    pub openai: Vec<DailyData>,
    pub anthropic: Vec<DailyData>,
    pub anthropic_usage: Vec<DailyUsageData>,
    pub openai_usage: Vec<DailyUsageData>,
    pub anthropic_api_key_names: std::collections::HashMap<String, String>,
    pub openai_api_key_names: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct OpenAICostResponse {
    pub data: Vec<OpenAIBucket<OpenAICostResult>>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIBucket<T> {
    pub start_time: i64,
    pub results: Vec<T>,
}

#[derive(Deserialize)]
pub struct OpenAICostResult {
    pub amount: OpenAICostAmount,
    #[serde(default)]
    pub line_item: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAICostAmount {
    pub value: f64,
}

#[derive(Deserialize)]
pub struct AnthropicCostResponse {
    pub data: Vec<AnthropicCostBucket>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

#[derive(Deserialize)]
pub struct AnthropicCostBucket {
    pub starting_at: String,
    pub results: Vec<AnthropicCostResult>,
}

#[derive(Deserialize)]
pub struct AnthropicCostResult {
    #[allow(dead_code)]
    pub currency: String,
    pub amount: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub cost_type: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub context_window: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub service_tier: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub token_type: Option<String>,
}

#[derive(Deserialize)]
pub struct AnthropicUsageResponse {
    pub data: Vec<AnthropicUsageTimeBucket>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

#[derive(Deserialize)]
pub struct AnthropicUsageTimeBucket {
    pub starting_at: String,
    #[allow(dead_code)]
    pub ending_at: String,
    pub results: Vec<AnthropicUsageItem>,
}

#[derive(Deserialize)]
pub struct AnthropicUsageItem {
    pub uncached_input_tokens: u64,
    pub cache_creation: CacheCreation,
    pub cache_read_input_tokens: u64,
    pub output_tokens: u64,
    #[allow(dead_code)]
    pub server_tool_use: ServerToolUse,
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub service_tier: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub context_window: Option<String>,
}

#[derive(Deserialize)]
pub struct CacheCreation {
    pub ephemeral_1h_input_tokens: u64,
    pub ephemeral_5m_input_tokens: u64,
}

#[derive(Deserialize)]
pub struct ServerToolUse {
    #[serde(default)]
    #[allow(dead_code)]
    pub web_search_requests: u64,
}

#[derive(Deserialize)]
pub struct OpenAIUsageResponse {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    pub data: Vec<OpenAIUsageBucket>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIUsageBucket {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    pub start_time: i64,
    #[allow(dead_code)]
    pub end_time: i64,
    pub results: Vec<OpenAIUsageResult>,
}

#[derive(Deserialize)]
pub struct OpenAIUsageResult {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    // For completions
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub input_cached_tokens: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub input_audio_tokens: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub output_audio_tokens: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub num_model_requests: u64,
    // For embeddings (uses input_tokens above)
    // For images
    #[serde(default)]
    #[allow(dead_code)]
    pub images: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub project_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub api_key_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectsResponse {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    pub data: Vec<OpenAIProject>,
    #[serde(default)]
    #[allow(dead_code)]
    pub first_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub last_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Deserialize)]
pub struct OpenAIProject {
    pub id: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    #[allow(dead_code)]
    pub name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub created_at: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub archived_at: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKey {
    pub id: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub redacted_value: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub created_at: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub last_used_at: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub owner: Option<OpenAIProjectApiKeyOwner>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeyOwner {
    #[serde(default)]
    #[allow(dead_code)]
    pub r#type: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub user: Option<OpenAIProjectApiKeyOwnerUser>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeyOwnerUser {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub email: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub role: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub added_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeysResponse {
    #[serde(default)]
    #[allow(dead_code)]
    pub object: Option<String>,
    pub data: Vec<OpenAIProjectApiKey>,
    #[serde(default)]
    #[allow(dead_code)]
    pub first_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub last_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

impl UsageData {
    pub fn new() -> Self {
        Self {
            openai: Vec::new(),
            anthropic: Vec::new(),
            anthropic_usage: Vec::new(),
            openai_usage: Vec::new(),
            anthropic_api_key_names: std::collections::HashMap::new(),
            openai_api_key_names: std::collections::HashMap::new(),
        }
    }
    pub fn openai_total_cost(&self) -> f64 {
        self.openai.iter().map(|d| d.cost).sum()
    }
    pub fn anthropic_total_cost(&self) -> f64 {
        self.anthropic.iter().map(|d| d.cost).sum()
    }
    pub fn anthropic_total_input_tokens(&self) -> u64 {
        self.anthropic_usage.iter().map(|d| d.input_tokens).sum()
    }
    pub fn anthropic_total_output_tokens(&self) -> u64 {
        self.anthropic_usage.iter().map(|d| d.output_tokens).sum()
    }
    pub fn openai_total_input_tokens(&self) -> u64 {
        self.openai_usage.iter().map(|d| d.input_tokens).sum()
    }
    pub fn openai_total_output_tokens(&self) -> u64 {
        self.openai_usage.iter().map(|d| d.output_tokens).sum()
    }
}
