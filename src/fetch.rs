use crate::api::{anthropic::AnthropicClient, openai::OpenAIClient};
use crate::models::{DailyData, DailyUsageData};
use crate::provider::{Provider, ProviderErrors};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

pub async fn fetch_data(
    provider: Provider,
    openai_client: Option<OpenAIClient>,
    anthropic_client: Option<AnthropicClient>,
) -> crate::provider::FetchOutcome {
    match provider {
        Provider::OpenAI => fetch_openai_data(openai_client).await,
        Provider::Anthropic => fetch_anthropic_data(anthropic_client).await,
    }
}

fn usage_start_time() -> DateTime<Utc> {
    let now = Utc::now();
    (now.date_naive() - Duration::days(7))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

fn append_error(target: &mut Option<String>, message: String) {
    if let Some(existing) = target.take() {
        *target = Some(format!("{}; {}", existing, message));
    } else {
        *target = Some(message);
    }
}

async fn fetch_openai_data(client: Option<OpenAIClient>) -> crate::provider::FetchOutcome {
    let mut errors = ProviderErrors::default();
    let mut cost_data = Vec::new();
    let mut usage_data = Vec::new();
    let mut api_key_names = HashMap::new();
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
                        cost_data.push(DailyData {
                            date,
                            cost: result.amount.value,
                            line_item: result.line_item,
                        });
                    }
                }
                cost_data.sort_by_key(|d| d.date);
            }
            Err(e) => {
                append_error(&mut errors.cost, e.to_string());
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
                            usage_data.push(DailyUsageData {
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
                usage_data.sort_by_key(|d| d.date);

                let api_key_ids: Vec<String> = api_key_ids
                    .into_iter()
                    .filter(|id| !id.is_empty() && id != "unknown")
                    .collect();

                if !api_key_ids.is_empty() {
                    match client.fetch_api_key_names_for_ids(&api_key_ids).await {
                        Ok(api_key_map) => {
                            api_key_names.extend(api_key_map);
                        }
                        Err(e) => {
                            append_error(
                                &mut errors.usage,
                                format!("API key name fetch failed: {}", e),
                            );
                        }
                    }
                }
            }
            Err(e) => {
                append_error(&mut errors.usage, format!("Usage fetch failed: {}", e));
            }
        }
    }

    crate::provider::FetchOutcome {
        provider: Provider::OpenAI,
        cost_data,
        usage_data,
        api_key_names,
        errors,
    }
}

async fn fetch_anthropic_data(client: Option<AnthropicClient>) -> crate::provider::FetchOutcome {
    let mut errors = ProviderErrors::default();
    let mut cost_data = Vec::new();
    let mut usage_data = Vec::new();
    let mut api_key_names = HashMap::new();
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
                                    cost_data.push(DailyData {
                                        date,
                                        cost,
                                        line_item: result.model,
                                    });
                                }
                            }
                        }
                    }
                }
                cost_data.sort_by_key(|d| d.date);
            }
            Err(e) => {
                append_error(&mut errors.cost, e.to_string());
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
                                usage_data.push(DailyUsageData {
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
                usage_data.sort_by_key(|d| d.date);

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
                                api_key_names.insert(api_key_id, name);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                append_error(&mut errors.usage, format!("Usage fetch failed: {}", e));
            }
        }
    }

    crate::provider::FetchOutcome {
        provider: Provider::Anthropic,
        cost_data,
        usage_data,
        api_key_names,
        errors,
    }
}
