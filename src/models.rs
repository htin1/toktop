use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone)]
pub struct DailyData {
    pub date: DateTime<Utc>,
    pub cost: f64,
    pub line_item: Option<String>,
}

#[derive(Clone)]
pub struct UsageData {
    pub openai: Vec<DailyData>,
    pub anthropic: Vec<DailyData>,
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

impl UsageData {
    pub fn new() -> Self {
        Self {
            openai: Vec::new(),
            anthropic: Vec::new(),
        }
    }
    pub fn openai_total_cost(&self) -> f64 {
        self.openai.iter().map(|d| d.cost).sum()
    }
    pub fn anthropic_total_cost(&self) -> f64 {
        self.anthropic.iter().map(|d| d.cost).sum()
    }
}
