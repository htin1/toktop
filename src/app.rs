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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    Model,
    ApiKeys,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OptionsColumn {
    Provider,
    Metric,
    GroupBy,
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
    pub options_column: OptionsColumn,
    pub current_view: View,
    pub group_by: GroupBy,
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
            options_column: OptionsColumn::Provider,
            current_view: View::Usage,
            group_by: GroupBy::Model,
            openai_error: None,
            anthropic_error: None,
            api_key_popup_active: None,
            api_key_input: String::new(),
            animation_frame: 0,
        }
    }

    pub fn move_options_column(&mut self, delta: isize) {
        let columns = [
            OptionsColumn::Provider,
            OptionsColumn::Metric,
            OptionsColumn::GroupBy,
        ];
        let len = columns.len() as isize;
        let current_idx = columns
            .iter()
            .position(|&c| c == self.options_column)
            .unwrap_or(0) as isize;
        let next = (current_idx + delta).rem_euclid(len);
        self.options_column = columns[next as usize];
    }

    pub fn move_column_cursor(&mut self, delta: isize) {
        match self.options_column {
            OptionsColumn::Provider => {
                let providers = [Provider::OpenAI, Provider::Anthropic];
                let len = providers.len() as isize;
                if let Some(idx) = providers
                    .iter()
                    .position(|&provider| provider == self.selected_provider)
                {
                    let next = (idx as isize + delta).rem_euclid(len);
                    let new_provider = providers[next as usize];
                    if new_provider != self.selected_provider {
                        self.selected_provider = new_provider;
                        if !self.has_client(new_provider) {
                            self.show_api_key_popup(new_provider);
                        } else {
                            self.cancel_api_key_popup();
                        }
                    }
                }
            }
            OptionsColumn::Metric => {
                let metrics = [View::Usage, View::Cost];
                let len = metrics.len() as isize;
                if let Some(idx) = metrics.iter().position(|&view| view == self.current_view) {
                    let next = (idx as isize + delta).rem_euclid(len);
                    let new_view = metrics[next as usize];
                    if new_view != self.current_view {
                        self.current_view = new_view;
                        // When switching to Cost, ensure group_by is Model
                        if self.current_view == View::Cost {
                            self.group_by = GroupBy::Model;
                        }
                    }
                }
            }
            OptionsColumn::GroupBy => {
                if self.current_view == View::Usage {
                    let group_by_options = [GroupBy::Model, GroupBy::ApiKeys];
                    let len = group_by_options.len() as isize;
                    if let Some(idx) = group_by_options
                        .iter()
                        .position(|&group| group == self.group_by)
                    {
                        let next = (idx as isize + delta).rem_euclid(len);
                        self.group_by = group_by_options[next as usize];
                    }
                }
            }
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

pub async fn fetch_data(
    provider: Provider,
    openai_client: Option<OpenAIClient>,
    anthropic_client: Option<AnthropicClient>,
) -> FetchOutcome {
    let mut usage_data = UsageData::new();
    let mut openai_error = None;
    let mut anthropic_error = None;

    match provider {
        Provider::OpenAI => {
            openai_error = fetch_openai_data(openai_client, &mut usage_data).await;
        }
        Provider::Anthropic => {
            anthropic_error = fetch_anthropic_data(anthropic_client, &mut usage_data).await;
        }
    }

    FetchOutcome {
        data: usage_data,
        openai_error,
        anthropic_error,
    }
}

fn usage_start_time() -> DateTime<Utc> {
    let now = Utc::now();
    (now.date_naive() - Duration::days(7))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

async fn fetch_openai_data(
    client: Option<OpenAIClient>,
    usage_data: &mut UsageData,
) -> Option<String> {
    let mut openai_error = None;
    let start_time = usage_start_time();

    if let Some(client) = client {
        let (costs_result, usage_result) =
            tokio::join!(client.fetch_costs(), client.fetch_usage(start_time),);

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
                        usage_data.openai.push(DailyData {
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

        match usage_result {
            Ok(buckets) => {
                let mut api_key_ids = std::collections::HashSet::new();

                for bucket in &buckets {
                    let date = DateTime::from_timestamp(bucket.start_time, 0)
                        .unwrap_or(Utc::now() - Duration::days(7))
                        .date_naive()
                        .and_hms_opt(0, 0, 0)
                        .unwrap();
                    let date = DateTime::<Utc>::from_naive_utc_and_offset(date, Utc);

                    for result in &bucket.results {
                        let input_tokens = result.input_tokens;
                        let output_tokens = result.output_tokens;

                        if input_tokens > 0 || output_tokens > 0 {
                            usage_data.openai_usage.push(DailyUsageData {
                                date,
                                input_tokens,
                                output_tokens,
                                api_key_id: result.api_key_id.clone(),
                                model: result.model.clone(),
                            });

                            if let Some(ref api_key_id) = result.api_key_id {
                                api_key_ids.insert(api_key_id.clone());
                            }
                        }
                    }
                }
                usage_data.openai_usage.sort_by_key(|d| d.date);

                let api_key_ids: Vec<String> = api_key_ids
                    .into_iter()
                    .filter(|id| !id.is_empty() && id != "unknown")
                    .collect();

                if !api_key_ids.is_empty() {
                    match client.fetch_api_key_names_for_ids(&api_key_ids).await {
                        Ok(api_key_map) => {
                            usage_data.openai_api_key_names.extend(api_key_map);
                        }
                        Err(e) => {
                            let message = format!("API key name fetch failed: {}", e);
                            if let Some(existing) = openai_error.take() {
                                openai_error = Some(format!("{}; {}", existing, message));
                            } else {
                                openai_error = Some(message);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let message = format!("Usage fetch failed: {}", e);
                if let Some(existing) = openai_error.take() {
                    openai_error = Some(format!("{}; {}", existing, message));
                } else {
                    openai_error = Some(message);
                }
            }
        }
    }

    openai_error
}

async fn fetch_anthropic_data(
    client: Option<AnthropicClient>,
    usage_data: &mut UsageData,
) -> Option<String> {
    let mut anthropic_error = None;
    let start_time = usage_start_time();

    if let Some(client) = client {
        let (costs_result, usage_result) = tokio::join!(
            client.fetch_costs(start_time),
            client.fetch_usage(start_time),
        );

        match costs_result {
            Ok(buckets) => {
                for bucket in buckets {
                    if let Ok(bucket_start) = DateTime::parse_from_rfc3339(&bucket.starting_at) {
                        let date = bucket_start.with_timezone(&Utc);
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

        match usage_result {
            Ok(buckets) => {
                let mut api_key_ids = std::collections::HashSet::new();

                for bucket in &buckets {
                    if let Ok(bucket_start) = DateTime::parse_from_rfc3339(&bucket.starting_at) {
                        let date = bucket_start.with_timezone(&Utc);
                        for result in &bucket.results {
                            let input_tokens = result.uncached_input_tokens
                                + result.cache_creation.ephemeral_1h_input_tokens
                                + result.cache_creation.ephemeral_5m_input_tokens
                                + result.cache_read_input_tokens;

                            if input_tokens > 0 || result.output_tokens > 0 {
                                usage_data.anthropic_usage.push(DailyUsageData {
                                    date,
                                    input_tokens,
                                    output_tokens: result.output_tokens,
                                    api_key_id: result.api_key_id.clone(),
                                    model: result.model.clone(),
                                });

                                if let Some(ref api_key_id) = result.api_key_id {
                                    api_key_ids.insert(api_key_id.clone());
                                }
                            }
                        }
                    }
                }
                usage_data.anthropic_usage.sort_by_key(|d| d.date);

                let api_key_ids: Vec<String> = api_key_ids
                    .into_iter()
                    .filter(|id| !id.is_empty() && id != "unknown")
                    .collect();

                if !api_key_ids.is_empty() {
                    let name_futures: Vec<_> = api_key_ids
                        .into_iter()
                        .map(|api_key_id| {
                            let client_clone = client.clone();
                            tokio::spawn(async move {
                                let result = client_clone.fetch_api_key_name(&api_key_id).await;
                                (api_key_id, result)
                            })
                        })
                        .collect();

                    for handle in name_futures {
                        if let Ok((api_key_id, result)) = handle.await {
                            if let Ok(name) = result {
                                usage_data.anthropic_api_key_names.insert(api_key_id, name);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let message = format!("Usage fetch failed: {}", e);
                if let Some(existing) = anthropic_error.take() {
                    anthropic_error = Some(format!("{}; {}", existing, message));
                } else {
                    anthropic_error = Some(message);
                }
            }
        }
    }

    anthropic_error
}
