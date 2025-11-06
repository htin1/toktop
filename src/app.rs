use crate::api::{anthropic::AnthropicClient, openai::OpenAIClient};
use crate::models::{DailyData, UsageData};
use chrono::{DateTime, Duration, Utc};
use std::io::{self, Write};

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
    pub openai_error: Option<String>,
    pub anthropic_error: Option<String>,
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
            openai_error: None,
            anthropic_error: None,
        }
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
}

pub async fn fetch_usage_data(
    provider: Provider,
    openai_client: Option<OpenAIClient>,
    anthropic_client: Option<AnthropicClient>,
) -> FetchOutcome {
    let mut usage_data = UsageData::new();
    let mut openai_error = None;
    let mut anthropic_error = None;

    match provider {
        Provider::OpenAI => {
            if let Some(client) = openai_client {
                match client.fetch_costs().await {
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
            }
        }
        Provider::Anthropic => {
            if let Some(client) = anthropic_client {
                let start_time = Utc::now() - Duration::days(7);
                match client.fetch_costs(start_time).await {
                    Ok(buckets) => {
                        for bucket in buckets {
                            if let Ok(bucket_start) = DateTime::parse_from_rfc3339(&bucket.starting_at) {
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
            }
        }
    }

    FetchOutcome {
        data: usage_data,
        openai_error,
        anthropic_error,
    }
}

pub fn prompt_for_key(service: &str) -> String {
    print!("Enter {} Admin API Key: ", service);
    io::stdout().flush().unwrap();
    let mut key = String::new();
    io::stdin().read_line(&mut key).unwrap();
    key.trim().to_string()
}
