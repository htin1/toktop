use crate::api::{anthropic::AnthropicClient, openai::OpenAIClient};
use crate::models::{DailyData, DailyUsageData, UsageData};
use chrono::{DateTime, Duration, Utc};

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Cost,
    Usage,
}

pub struct FetchOutcome {
    pub data: UsageData,
    pub openai_error: Option<String>,
    pub anthropic_error: Option<String>,
}

pub struct App {
    pub openai_client: Option<OpenAIClient>,
    pub anthropic_client: Option<AnthropicClient>,
    pub data: UsageData,
    pub loading: bool,
    pub selected_provider: Provider,
    pub menu_cursor: usize,
    pub current_view: View,
    pub openai_error: Option<String>,
    pub anthropic_error: Option<String>,
    pub api_key_popup_active: Option<Provider>,
    pub api_key_input: String,
    pub animation_frame: u32,
}

impl App {
    pub fn new() -> Self {
        Self {
            openai_client: None,
            anthropic_client: None,
            data: UsageData::new(),
            loading: false,
            selected_provider: Provider::OpenAI,
            menu_cursor: 0,
            current_view: View::Cost,
            openai_error: None,
            anthropic_error: None,
            api_key_popup_active: None,
            api_key_input: String::new(),
            animation_frame: 0,
        }
    }

    pub fn toggle_view(&mut self) {
        self.current_view = match self.current_view {
            View::Cost => View::Usage,
            View::Usage => View::Cost,
        };
    }

    pub fn set_openai_client(&mut self, api_key: String) {
        self.openai_client = Some(OpenAIClient::new(api_key));
        self.ensure_selection_has_client();
    }

    pub fn set_anthropic_client(&mut self, api_key: String) {
        self.anthropic_client = Some(AnthropicClient::new(api_key));
        self.ensure_selection_has_client();
    }

    pub fn current_provider(&self) -> Provider {
        self.selected_provider
    }

    pub fn move_menu_cursor(&mut self, delta: isize) {
        let providers = [Provider::OpenAI, Provider::Anthropic];
        let len = providers.len() as isize;
        let cursor = self.menu_cursor as isize;
        let next = (cursor + delta).rem_euclid(len);
        self.menu_cursor = next as usize;
    }

    pub fn select_menu_cursor(&mut self) -> bool {
        let providers = [Provider::OpenAI, Provider::Anthropic];
        if self.menu_cursor < providers.len() {
            let new_provider = providers[self.menu_cursor];
            if new_provider != self.selected_provider {
                self.selected_provider = new_provider;
                return true; // Provider changed
            }
        }
        false // Provider didn't change
    }

    pub fn ensure_selection_has_client(&mut self) {
        if self.has_client(self.selected_provider) {
            return;
        }
        // Switch to first available provider
        if self.has_client(Provider::OpenAI) {
            self.selected_provider = Provider::OpenAI;
        } else if self.has_client(Provider::Anthropic) {
            self.selected_provider = Provider::Anthropic;
        }
    }

    pub fn has_client(&self, provider: Provider) -> bool {
        match provider {
            Provider::OpenAI => self.openai_client.is_some(),
            Provider::Anthropic => self.anthropic_client.is_some(),
        }
    }

    pub fn error_for_provider(&self, provider: Provider) -> Option<&String> {
        match provider {
            Provider::OpenAI => self.openai_error.as_ref(),
            Provider::Anthropic => self.anthropic_error.as_ref(),
        }
    }

    pub fn data_for_provider(&self, provider: Provider) -> Option<&[DailyData]> {
        match provider {
            Provider::OpenAI => Some(&self.data.openai),
            Provider::Anthropic => Some(&self.data.anthropic),
        }
    }

    pub fn usage_data_for_provider(&self, provider: Provider) -> Option<&[DailyUsageData]> {
        match provider {
            Provider::Anthropic => Some(&self.data.anthropic_usage),
            Provider::OpenAI => Some(&self.data.openai_usage),
        }
    }

    pub fn show_api_key_popup(&mut self, provider: Provider) {
        self.api_key_popup_active = Some(provider);
        self.api_key_input.clear();
    }

    pub fn cancel_api_key_popup(&mut self) {
        self.api_key_popup_active = None;
        self.api_key_input.clear();
    }

    pub fn submit_api_key(&mut self) -> bool {
        if let Some(provider) = self.api_key_popup_active {
            let key = self.api_key_input.trim().to_string();
            if !key.is_empty() {
                match provider {
                    Provider::OpenAI => {
                        self.set_openai_client(key);
                    }
                    Provider::Anthropic => {
                        self.set_anthropic_client(key);
                    }
                }
                self.api_key_popup_active = None;
                self.api_key_input.clear();
                return true; // Key was submitted
            }
        }
        false
    }

    pub fn handle_api_key_input(&mut self, key_code: crossterm::event::KeyCode) {
        match key_code {
            crossterm::event::KeyCode::Char(c) => {
                self.api_key_input.push(c);
            }
            crossterm::event::KeyCode::Backspace => {
                self.api_key_input.pop();
            }
            _ => {}
        }
    }
}

pub async fn fetch_usage_data(
    provider: Provider,
    openai_client: Option<OpenAIClient>,
    anthropic_client: Option<AnthropicClient>,
) -> FetchOutcome {
    let mut usage_data = UsageData::new();
    let mut openai_error = None;
    let mut anthropic_error = None;

    // Set start_time to start of day (midnight UTC) 7 days ago
    // This ensures we get exactly 7 full days of data
    let now = Utc::now();
    let start_time = (now.date_naive() - Duration::days(7))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    match provider {
        Provider::OpenAI => {
            if let Some(client) = openai_client {
                // Fetch cost and usage data in parallel
                let (costs_result, usage_result) =
                    tokio::join!(client.fetch_costs(), client.fetch_usage(start_time),);

                // Process cost data
                match costs_result {
                    Ok(buckets) => {
                        for bucket in buckets {
                            let date = DateTime::from_timestamp(bucket.start_time, 0)
                                .unwrap_or(Utc::now() - Duration::days(7))
                                .date_naive()
                                .and_hms_opt(0, 0, 0)
                                .unwrap();
                            let date = DateTime::<Utc>::from_naive_utc_and_offset(date, Utc);

                            for result in bucket.results {
                                usage_data.openai.push(crate::models::DailyData {
                                    date,
                                    cost: result.amount.value,
                                    line_item: result.line_item,
                                });
                            }
                        }
                        usage_data.openai.sort_by_key(|d| d.date);
                    }
                    Err(e) => {
                        openai_error = Some(e.to_string());
                    }
                }

                // Process usage data
                match usage_result {
                    Ok(buckets) => {
                        for bucket in buckets {
                            let date = DateTime::from_timestamp(bucket.start_time, 0)
                                .unwrap_or(Utc::now() - Duration::days(7))
                                .date_naive()
                                .and_hms_opt(0, 0, 0)
                                .unwrap();
                            let date = DateTime::<Utc>::from_naive_utc_and_offset(date, Utc);

                            for result in bucket.results {
                                // For completions: input_tokens and output_tokens are present
                                // For embeddings: only input_tokens is present (no output_tokens)
                                // For images: images field is present (no tokens)
                                let input_tokens = result.input_tokens;
                                let output_tokens = result.output_tokens;

                                // Only add if there are actual tokens (skip images which don't have tokens)
                                if input_tokens > 0 || output_tokens > 0 {
                                    usage_data.openai_usage.push(DailyUsageData {
                                        date,
                                        input_tokens,
                                        output_tokens,
                                        api_key_id: result.api_key_id.clone(),
                                        model: result.model.clone(),
                                    });
                                }
                            }
                        }
                        usage_data.openai_usage.sort_by_key(|d| d.date);
                    }
                    Err(e) => {
                        // Don't overwrite cost error, but append usage error if cost succeeded
                        if openai_error.is_none() {
                            openai_error = Some(format!("Usage fetch failed: {}", e));
                        } else {
                            let cost_err = openai_error.clone().unwrap();
                            openai_error = Some(format!("{}; Usage fetch failed: {}", cost_err, e));
                        }
                    }
                }
            }
        }
        Provider::Anthropic => {
            if let Some(client) = anthropic_client {
                // Fetch cost and usage data in parallel
                let (costs_result, usage_result) = tokio::join!(
                    client.fetch_costs(start_time),
                    client.fetch_usage(start_time),
                );

                // Process cost data
                match costs_result {
                    Ok(buckets) => {
                        for bucket in buckets {
                            if let Ok(bucket_start) =
                                DateTime::parse_from_rfc3339(&bucket.starting_at)
                            {
                                let date = bucket_start.with_timezone(&Utc);
                                // Create a separate entry for each result (grouped by model)
                                // Amount is in cents (lowest currency units), so divide by 100
                                for result in bucket.results {
                                    if let Ok(cost_cents) = result.amount.parse::<f64>() {
                                        let cost = cost_cents / 100.0;
                                        if cost > 0.0 {
                                            usage_data.anthropic.push(DailyData {
                                                date,
                                                cost,
                                                line_item: result.model,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        usage_data.anthropic.sort_by_key(|d| d.date);
                    }
                    Err(e) => {
                        anthropic_error = Some(e.to_string());
                    }
                }

                // Process usage data
                match usage_result {
                    Ok(buckets) => {
                        for bucket in buckets {
                            if let Ok(bucket_start) =
                                DateTime::parse_from_rfc3339(&bucket.starting_at)
                            {
                                let date = bucket_start.with_timezone(&Utc);
                                for result in bucket.results {
                                    // Calculate total input tokens
                                    let input_tokens = result.uncached_input_tokens
                                        + result.cache_creation.ephemeral_1h_input_tokens
                                        + result.cache_creation.ephemeral_5m_input_tokens
                                        + result.cache_read_input_tokens;

                                    if input_tokens > 0 || result.output_tokens > 0 {
                                        usage_data.anthropic_usage.push(DailyUsageData {
                                            date,
                                            input_tokens,
                                            output_tokens: result.output_tokens,
                                            api_key_id: None,
                                            model: result.model.clone(),
                                        });
                                    }
                                }
                            }
                        }
                        usage_data.anthropic_usage.sort_by_key(|d| d.date);
                    }
                    Err(e) => {
                        // Don't overwrite cost error, but append usage error if cost succeeded
                        if anthropic_error.is_none() {
                            anthropic_error = Some(format!("Usage fetch failed: {}", e));
                        } else {
                            let cost_err = anthropic_error.clone().unwrap();
                            anthropic_error =
                                Some(format!("{}; Usage fetch failed: {}", cost_err, e));
                        }
                    }
                }
            }
        }
    }

    FetchOutcome {
        data: usage_data,
        openai_error,
        anthropic_error,
    }
}
