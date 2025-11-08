use crate::api::{anthropic::AnthropicClient, openai::OpenAIClient};
use crate::models::{DailyData, DailyUsageData};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAI,
    Anthropic,
}

impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
        }
    }
}

#[derive(Default, Clone)]
pub struct ProviderErrors {
    pub cost: Option<String>,
    pub usage: Option<String>,
}

#[derive(Clone)]
pub enum ProviderClient {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
}

pub struct ProviderInfo {
    pub client: Option<ProviderClient>,
    pub errors: ProviderErrors,
    pub initial_fetch_done: bool,
    pub cost_data: Vec<DailyData>,
    pub usage_data: Vec<DailyUsageData>,
    pub api_key_names: HashMap<String, String>,
}

impl ProviderInfo {
    pub fn new() -> Self {
        Self {
            client: None,
            errors: ProviderErrors::default(),
            initial_fetch_done: false,
            cost_data: Vec::new(),
            usage_data: Vec::new(),
            api_key_names: HashMap::new(),
        }
    }
}

pub struct FetchOutcome {
    pub provider: Provider,
    pub cost_data: Vec<DailyData>,
    pub usage_data: Vec<DailyUsageData>,
    pub api_key_names: HashMap<String, String>,
    pub errors: ProviderErrors,
}
