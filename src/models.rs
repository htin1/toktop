use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AnthropicApiKeyResponse {
    #[serde(skip)]
    #[expect(unused)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    #[expect(unused)]
    pub status: Option<String>,
    #[serde(default)]
    #[expect(unused)]
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
    // Cache metrics (Anthropic only)
    pub cache_read_input_tokens: Option<u64>,
    pub uncached_input_tokens: Option<u64>,
    // Request count (OpenAI only)
    pub num_requests: Option<u64>,
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
    #[serde(skip)]
    #[expect(unused)]
    pub currency: String,
    pub amount: String,
    #[serde(default)]
    #[expect(unused)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub description: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub cost_type: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub context_window: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub service_tier: Option<String>,
    #[serde(default)]
    #[expect(unused)]
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
    #[serde(skip)]
    #[expect(unused)]
    pub ending_at: String,
    pub results: Vec<AnthropicUsageItem>,
}

#[derive(Deserialize)]
pub struct AnthropicUsageItem {
    pub uncached_input_tokens: u64,
    pub cache_creation: CacheCreation,
    pub cache_read_input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    #[expect(unused)]
    pub server_tool_use: ServerToolUse,
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub service_tier: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub context_window: Option<String>,
}

#[derive(Deserialize)]
pub struct CacheCreation {
    pub ephemeral_1h_input_tokens: u64,
    pub ephemeral_5m_input_tokens: u64,
}

#[derive(Deserialize, Default)]
pub struct ServerToolUse {
    #[serde(default)]
    #[expect(unused)]
    pub web_search_requests: u64,
}

#[derive(Deserialize)]
pub struct OpenAIUsageResponse {
    #[serde(default)]
    #[expect(unused)]
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
    #[expect(unused)]
    pub object: Option<String>,
    pub start_time: i64,
    #[serde(skip)]
    #[expect(unused)]
    pub end_time: i64,
    pub results: Vec<OpenAIUsageResult>,
}

#[derive(Deserialize)]
pub struct OpenAIUsageResult {
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    // For completions
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    #[expect(unused)]
    pub input_cached_tokens: u64,
    #[serde(default)]
    #[expect(unused)]
    pub input_audio_tokens: u64,
    #[serde(default)]
    #[expect(unused)]
    pub output_audio_tokens: u64,
    #[serde(default)]
    pub num_model_requests: u64,
    // For embeddings (uses input_tokens above)
    // For images
    #[serde(default)]
    #[expect(unused)]
    pub images: u64,
    #[serde(default)]
    #[expect(unused)]
    pub project_id: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub api_key_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectsResponse {
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    pub data: Vec<OpenAIProject>,
    #[serde(default)]
    #[expect(unused)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Deserialize)]
pub struct OpenAIProject {
    pub id: String,
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    #[serde(skip)]
    #[expect(unused)]
    pub name: String,
    #[serde(default)]
    #[expect(unused)]
    pub created_at: Option<i64>,
    #[serde(default)]
    #[expect(unused)]
    pub archived_at: Option<i64>,
    #[serde(default)]
    #[expect(unused)]
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKey {
    pub id: String,
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub redacted_value: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub created_at: Option<i64>,
    #[serde(default)]
    #[expect(unused)]
    pub last_used_at: Option<i64>,
    #[serde(default)]
    #[expect(unused)]
    pub owner: Option<OpenAIProjectApiKeyOwner>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeyOwner {
    #[serde(default)]
    #[expect(unused)]
    pub r#type: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub user: Option<OpenAIProjectApiKeyOwnerUser>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeyOwnerUser {
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub id: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub name: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub email: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub role: Option<String>,
    #[serde(default)]
    #[expect(unused)]
    pub added_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct OpenAIProjectApiKeysResponse {
    #[serde(default)]
    #[expect(unused)]
    pub object: Option<String>,
    pub data: Vec<OpenAIProjectApiKey>,
    #[serde(default)]
    #[expect(unused)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}
