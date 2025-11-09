use crate::api::{anthropic::AnthropicClient, openai::OpenAIClient};
use crate::models::{DailyData, DailyUsageData};
use crate::provider::{Provider, ProviderClient, ProviderInfo};
use chrono::Duration;
use std::collections::HashMap;

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
pub enum Range {
    SevenDays,
    ThirtyDays,
}

impl Range {
    pub fn label(self) -> &'static str {
        match self {
            Range::SevenDays => "7d",
            Range::ThirtyDays => "30d",
        }
    }

    pub fn days(self) -> i64 {
        match self {
            Range::SevenDays => 7,
            Range::ThirtyDays => 30,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OptionsColumn {
    Provider,
    Metric,
    GroupBy,
    Range,
}

pub struct App {
    pub providers: HashMap<Provider, ProviderInfo>,
    pub loading: bool,
    pub selected_provider: Provider,
    pub options_column: OptionsColumn,
    pub current_view: View,
    pub group_by: GroupBy,
    pub range: Range,
    pub api_key_popup_active: Option<Provider>,
    pub api_key_input: String,
    pub animation_frame: u32,
    pub group_by_expanded: bool,
    pub selected_filter: Option<String>,
    pub filter_cursor_index: usize,
    pub chart_scrollbar_visible: bool,
}

impl App {
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        providers.insert(Provider::OpenAI, ProviderInfo::new());
        providers.insert(Provider::Anthropic, ProviderInfo::new());
        Self {
            providers,
            loading: false,
            selected_provider: Provider::OpenAI,
            options_column: OptionsColumn::Provider,
            current_view: View::Usage,
            group_by: GroupBy::Model,
            range: Range::SevenDays,
            api_key_popup_active: None,
            api_key_input: String::new(),
            animation_frame: 0,
            group_by_expanded: false,
            selected_filter: None,
            filter_cursor_index: 0,
            chart_scrollbar_visible: false,
        }
    }

    pub fn move_options_column(&mut self, delta: isize) {
        let columns = [
            OptionsColumn::Provider,
            OptionsColumn::Metric,
            OptionsColumn::GroupBy,
            OptionsColumn::Range,
        ];
        let len = columns.len() as isize;
        let current_idx = columns
            .iter()
            .position(|&c| c == self.options_column)
            .unwrap_or(0) as isize;
        let next = (current_idx + delta).rem_euclid(len);
        let new_column = columns[next as usize];
        if new_column != self.options_column {
            self.group_by_expanded = false;
        }
        self.options_column = new_column;
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
                        self.reset_filter();
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
                        if self.current_view == View::Cost {
                            self.group_by = GroupBy::Model;
                            self.reset_filter();
                        }
                    }
                }
            }
            OptionsColumn::GroupBy => {
                if self.group_by_expanded {
                    let filters = self.get_available_filters();
                    let total_items = filters.len() + 1;
                    if total_items > 0 {
                        let len = total_items as isize;
                        let current_idx = self.filter_cursor_index as isize;
                        let next = (current_idx + delta).rem_euclid(len);
                        self.filter_cursor_index = next as usize;

                        if self.filter_cursor_index == 0 {
                            self.selected_filter = None;
                        } else {
                            self.selected_filter =
                                filters.get(self.filter_cursor_index - 1).cloned();
                        }
                    }
                } else {
                    if self.current_view == View::Usage {
                        let group_by_options = [GroupBy::Model, GroupBy::ApiKeys];
                        let len = group_by_options.len() as isize;
                        if let Some(idx) = group_by_options
                            .iter()
                            .position(|&group| group == self.group_by)
                        {
                            let next = (idx as isize + delta).rem_euclid(len);
                            let new_group_by = group_by_options[next as usize];
                            if new_group_by != self.group_by {
                                self.group_by = new_group_by;
                                self.selected_filter = None;
                                self.filter_cursor_index = 0;
                            }
                        }
                    }
                }
            }
            OptionsColumn::Range => {
                let ranges = [Range::SevenDays, Range::ThirtyDays];
                let len = ranges.len() as isize;
                if let Some(idx) = ranges.iter().position(|&r| r == self.range) {
                    let next = (idx as isize + delta).rem_euclid(len);
                    self.range = ranges[next as usize];
                }
            }
        }
    }

    pub fn scroll_chart(&mut self, delta: isize) {
        let provider = self.current_provider();
        let current_view = self.current_view;
        let info = self.provider_info_mut(provider);
        let (scroll_value, data_len) = match current_view {
            View::Cost => (&mut info.cost_chart_scroll, info.cost_data.len()),
            View::Usage => (&mut info.usage_chart_scroll, info.usage_data.len()),
        };

        if delta == 0 || data_len == 0 {
            return;
        }

        if delta < 0 {
            let amount = delta.unsigned_abs() as usize;
            *scroll_value = scroll_value.saturating_sub(amount);
        } else {
            let amount = delta as usize;
            let max_position = data_len.saturating_sub(1);
            *scroll_value = scroll_value.saturating_add(amount).min(max_position);
        }
    }

    pub fn set_openai_client(&mut self, api_key: String) {
        let info = self.providers.get_mut(&Provider::OpenAI).unwrap();
        info.client = Some(ProviderClient::OpenAI(OpenAIClient::new(api_key)));
        info.initial_fetch_done = false;
        self.ensure_selection_has_client();
    }

    pub fn set_anthropic_client(&mut self, api_key: String) {
        let info = self.providers.get_mut(&Provider::Anthropic).unwrap();
        info.client = Some(ProviderClient::Anthropic(AnthropicClient::new(api_key)));
        info.initial_fetch_done = false;
        self.ensure_selection_has_client();
    }

    pub fn current_provider(&self) -> Provider {
        self.selected_provider
    }

    pub fn ensure_selection_has_client(&mut self) {
        if self.has_client(self.selected_provider) {
            return;
        }
        if self.has_client(Provider::OpenAI) {
            self.selected_provider = Provider::OpenAI;
        } else if self.has_client(Provider::Anthropic) {
            self.selected_provider = Provider::Anthropic;
        }
    }

    pub fn provider_info(&self, provider: Provider) -> &ProviderInfo {
        self.providers.get(&provider).unwrap()
    }

    pub fn provider_info_mut(&mut self, provider: Provider) -> &mut ProviderInfo {
        self.providers.get_mut(&provider).unwrap()
    }

    pub fn has_client(&self, provider: Provider) -> bool {
        self.provider_info(provider).client.is_some()
    }

    pub fn initial_fetch_done(&self, provider: Provider) -> bool {
        self.provider_info(provider).initial_fetch_done
    }

    pub fn mark_initial_fetch_done(&mut self, provider: Provider) {
        self.provider_info_mut(provider).initial_fetch_done = true;
    }

    pub fn error_for_provider(&self, provider: Provider, view: View) -> Option<&String> {
        let info = self.provider_info(provider);
        match view {
            View::Cost => info.errors.cost.as_ref(),
            View::Usage => info.errors.usage.as_ref(),
        }
    }

    pub fn data_for_provider(&self, provider: Provider) -> Option<&[DailyData]> {
        Some(&self.provider_info(provider).cost_data)
    }

    pub fn usage_data_for_provider(
        &self,
        provider: Provider,
    ) -> Option<&[crate::models::DailyUsageData]> {
        Some(&self.provider_info(provider).usage_data)
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
                return true;
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

    pub fn update_animation_frame(&mut self) {
        let provider = self.current_provider();
        let info = self.provider_info(provider);
        let has_data = !info.cost_data.is_empty() || !info.usage_data.is_empty();
        if self.loading || !has_data {
            self.animation_frame = self.animation_frame.wrapping_add(1);
        } else {
            self.animation_frame = 0;
        }
    }

    pub fn start_fetch(&mut self) {
        self.loading = true;
        let provider = self.current_provider();
        let info = self.provider_info_mut(provider);
        info.errors = crate::provider::ProviderErrors::default();
    }

    pub fn finish_fetch(&mut self, outcome: crate::provider::FetchOutcome) {
        let info = self.provider_info_mut(outcome.provider);
        info.cost_data = outcome.cost_data;
        info.usage_data = outcome.usage_data;
        info.api_key_names = outcome.api_key_names;
        info.errors = outcome.errors;
        self.mark_initial_fetch_done(outcome.provider);
        self.loading = false;
    }

    pub fn get_clients(&self) -> (Option<OpenAIClient>, Option<AnthropicClient>) {
        let openai_client = self
            .provider_info(Provider::OpenAI)
            .client
            .as_ref()
            .and_then(|c| match c {
                ProviderClient::OpenAI(client) => Some(client.clone()),
                _ => None,
            });
        let anthropic_client = self
            .provider_info(Provider::Anthropic)
            .client
            .as_ref()
            .and_then(|c| match c {
                ProviderClient::Anthropic(client) => Some(client.clone()),
                _ => None,
            });
        (openai_client, anthropic_client)
    }

    pub fn reset_filter(&mut self) {
        self.selected_filter = None;
        self.filter_cursor_index = 0;
        self.group_by_expanded = false;
    }

    pub fn toggle_group_by_expansion(&mut self) {
        if self.options_column == OptionsColumn::GroupBy {
            self.group_by_expanded = !self.group_by_expanded;
            if self.group_by_expanded {
                self.filter_cursor_index = 0;
                self.selected_filter = None;
            }
        }
    }

    pub fn get_available_filters(&self) -> Vec<String> {
        let provider = self.current_provider();
        let info = self.provider_info(provider);
        let filtered_usage_data = self.filter_usage_data_by_range(&info.usage_data);
        let filtered_cost_data = self.filter_cost_data_by_range(&info.cost_data);

        let filters: Vec<String> = match self.group_by {
            GroupBy::Model => {
                let mut models = std::collections::HashSet::new();
                for usage in &filtered_usage_data {
                    if let Some(ref model) = usage.model {
                        let model = model.trim();
                        if !model.is_empty() {
                            models.insert(model.to_string());
                        }
                    }
                }
                for cost in &filtered_cost_data {
                    if let Some(ref line_item) = cost.line_item {
                        let line_item = line_item.trim();
                        if !line_item.is_empty() {
                            models.insert(line_item.to_string());
                        }
                    }
                }
                models.into_iter().collect()
            }
            GroupBy::ApiKeys => {
                let mut api_keys = std::collections::HashSet::new();
                for usage in &filtered_usage_data {
                    if let Some(ref api_key_id) = usage.api_key_id {
                        let api_key_id = api_key_id.trim();
                        if !api_key_id.is_empty() {
                            api_keys.insert(api_key_id.to_string());
                        }
                    }
                }
                api_keys.into_iter().collect()
            }
        };

        let mut sorted_filters = filters;
        sorted_filters.sort();
        sorted_filters
    }

    fn filter_usage_data_by_range(&self, data: &[DailyUsageData]) -> Vec<DailyUsageData> {
        let latest_date = match data.iter().map(|d| d.date).max() {
            Some(date) => date,
            None => return Vec::new(),
        };
        let span = self.range.days().saturating_sub(1);
        let cutoff = latest_date - Duration::days(span);
        data.iter().filter(|d| d.date >= cutoff).cloned().collect()
    }

    fn filter_cost_data_by_range(&self, data: &[DailyData]) -> Vec<DailyData> {
        let latest_date = match data.iter().map(|d| d.date).max() {
            Some(date) => date,
            None => return Vec::new(),
        };
        let span = self.range.days().saturating_sub(1);
        let cutoff = latest_date - Duration::days(span);
        data.iter().filter(|d| d.date >= cutoff).cloned().collect()
    }
}
