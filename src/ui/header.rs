use crate::app::{App, Range};
use crate::models::{DailyData, DailyUsageData};
use crate::ui::banner;
use crate::ui::colors::ColorPalette;
use crate::ui::utils::format_tokens;
use chrono::{Duration, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    let info = app.provider_info(provider);
    let cost_data = &info.cost_data;
    let usage_data = &info.usage_data;
    let has_data = !cost_data.is_empty() || !usage_data.is_empty();

    if app.loading || !has_data {
        let mut text = vec![];
        text.extend(banner::render_animated_banner(app, &palette));
        f.render_widget(
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Summary")),
            area,
        );
        return;
    }

    let (total_cost, cost_bounds) = summarize_cost(&info.cost_data, app.range);
    let ((input_tokens, output_tokens), usage_bounds) =
        summarize_usage(&info.usage_data, app.range);
    let range_days = app.range.days().max(1) as f64;
    let avg_cost_per_day = total_cost / range_days;
    let total_tokens = input_tokens + output_tokens;
    let avg_tokens_per_day = total_tokens as f64 / range_days;

    // Calculate efficiency metrics
    let cost_per_million_tokens = calculate_cost_per_million_tokens(total_cost, total_tokens);
    let cache_hit_rate = calculate_cache_hit_rate(&info.usage_data, app.range);

    // Calculate period comparisons
    let cost_period_comparison = compare_cost_periods(&info.cost_data, app.range);
    let token_period_comparison = compare_token_periods(&info.usage_data, app.range);

    let date_range = {
        cost_bounds
            .or(usage_bounds)
            .map(|(min_date, max_date)| {
                format!(
                    "{} - {}",
                    min_date.format("%m/%d"),
                    max_date.format("%m/%d")
                )
            })
            .unwrap_or_else(|| "No data in selected range".to_string())
    };

    // Split into 2 columns
    let block = Block::default().borders(Borders::ALL).title("Summary");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left column: Cost & Token totals
    let mut left_text = vec![];
    left_text.push(Line::from(""));
    left_text.push(Line::from(vec![
        Span::styled(
            format!("Total Cost ({}): ", app.range.label()),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("${:.2}", total_cost),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    left_text.push(Line::from(vec![
        Span::styled("Average per day: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${:.2}", avg_cost_per_day),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(cost_per_mil) = cost_per_million_tokens {
        left_text.push(Line::from(vec![
            Span::styled("Cost per 1M tokens: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("${:.2}", cost_per_mil),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Cost period comparison (only for 7d range)
    if app.range == crate::app::Range::SevenDays {
        if let Some((change_pct, direction)) = cost_period_comparison {
            let change_color = if change_pct >= 0.0 {
                Color::Red
            } else {
                Color::Green
            };
            left_text.push(Line::from(vec![
                Span::styled("Change from last week: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} {:.1}%", direction, change_pct.abs()),
                    Style::default()
                        .fg(change_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }

    if total_tokens > 0 {
        left_text.push(Line::from(""));
        left_text.push(Line::from(vec![
            Span::styled(
                format!("Total Tokens ({}): ", app.range.label()),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format_tokens(total_tokens),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        left_text.push(Line::from(vec![
            Span::styled("Average per day: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format_tokens(avg_tokens_per_day as u64),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        left_text.push(Line::from(vec![
            Span::styled("Input: ", Style::default()),
            Span::styled(
                format_tokens(input_tokens),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Output: ", Style::default()),
            Span::styled(
                format_tokens(output_tokens),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Token period comparison (only for 7d range)
        if app.range == crate::app::Range::SevenDays {
            if let Some((change_pct, direction)) = token_period_comparison {
                let change_color = if change_pct >= 0.0 {
                    Color::Red
                } else {
                    Color::Green
                };
                left_text.push(Line::from(vec![
                    Span::styled("Change from last week: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{} {:.1}%", direction, change_pct.abs()),
                        Style::default()
                            .fg(change_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }
    }

    // Right column: Trends & Info
    let mut right_text = vec![];

    // Cache hit rate (Anthropic only)
    if let Some(hit_rate) = cache_hit_rate {
        right_text.push(Line::from(""));
        right_text.push(Line::from(vec![
            Span::styled("Cache hit rate: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", hit_rate),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    right_text.push(Line::from(""));
    right_text.push(Line::from(vec![
        Span::styled("Date Range: ", Style::default().fg(Color::Gray)),
        Span::raw(date_range),
    ]));

    f.render_widget(Paragraph::new(left_text), columns[0]);
    f.render_widget(Paragraph::new(right_text), columns[1]);
}

fn range_cutoff(range: Range, latest: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    let span = range.days().saturating_sub(1);
    latest - Duration::days(span)
}

fn summarize_cost(
    data: &[DailyData],
    range: Range,
) -> (f64, Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)>) {
    if data.is_empty() {
        return (0.0, None);
    }

    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return (0.0, None),
    };
    let cutoff = range_cutoff(range, latest);

    let mut total = 0.0;
    let mut min_date: Option<chrono::DateTime<Utc>> = None;
    let mut max_date: Option<chrono::DateTime<Utc>> = None;

    for entry in data {
        if entry.date >= cutoff {
            total += entry.cost;
            min_date = Some(min_date.map_or(entry.date, |min| min.min(entry.date)));
            max_date = Some(max_date.map_or(entry.date, |max| max.max(entry.date)));
        }
    }

    let bounds = min_date.zip(max_date);

    (total, bounds)
}

fn summarize_usage(
    data: &[DailyUsageData],
    range: Range,
) -> (
    (u64, u64),
    Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)>,
) {
    if data.is_empty() {
        return ((0, 0), None);
    }

    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return ((0, 0), None),
    };
    let cutoff = range_cutoff(range, latest);

    let mut input_total = 0;
    let mut output_total = 0;
    let mut min_date: Option<chrono::DateTime<Utc>> = None;
    let mut max_date: Option<chrono::DateTime<Utc>> = None;

    for entry in data {
        if entry.date >= cutoff {
            input_total += entry.input_tokens;
            output_total += entry.output_tokens;
            min_date = Some(min_date.map_or(entry.date, |min| min.min(entry.date)));
            max_date = Some(max_date.map_or(entry.date, |max| max.max(entry.date)));
        }
    }

    let bounds = min_date.zip(max_date);

    ((input_total, output_total), bounds)
}

fn calculate_cost_per_million_tokens(total_cost: f64, total_tokens: u64) -> Option<f64> {
    if total_tokens == 0 {
        return None;
    }
    let tokens_in_millions = total_tokens as f64 / 1_000_000.0;
    Some(total_cost / tokens_in_millions)
}

fn calculate_cache_hit_rate(
    usage_data: &[DailyUsageData],
    range: Range,
) -> Option<f64> {
    let latest = match usage_data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return None,
    };
    let cutoff = range_cutoff(range, latest);

    let mut cache_read_total = 0u64;
    let mut uncached_total = 0u64;

    for entry in usage_data {
        if entry.date >= cutoff {
            if let (Some(cache_read), Some(uncached)) = (
                entry.cache_read_input_tokens,
                entry.uncached_input_tokens,
            ) {
                cache_read_total += cache_read;
                uncached_total += uncached;
            }
        }
    }

    let total_cacheable = cache_read_total + uncached_total;
    if total_cacheable == 0 {
        return None;
    }

    let hit_rate = (cache_read_total as f64 / total_cacheable as f64) * 100.0;
    Some(hit_rate)
}

fn compare_cost_periods(cost_data: &[DailyData], range: Range) -> Option<(f64, String)> {
    if cost_data.is_empty() {
        return None;
    }

    let latest = match cost_data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return None,
    };

    let cutoff = range_cutoff(range, latest);
    let period_days = range.days() as i64;

    // Calculate current period total
    let current_cost: f64 = cost_data
        .iter()
        .filter(|d| d.date >= cutoff)
        .map(|d| d.cost)
        .sum();

    // Calculate previous period total
    let previous_cutoff = cutoff - Duration::days(period_days);
    let previous_cost: f64 = cost_data
        .iter()
        .filter(|d| d.date >= previous_cutoff && d.date < cutoff)
        .map(|d| d.cost)
        .sum();

    if previous_cost == 0.0 {
        return None;
    }

    let change_pct = ((current_cost - previous_cost) / previous_cost) * 100.0;
    let direction = if change_pct >= 0.0 { "↑" } else { "↓" };

    Some((change_pct, direction.to_string()))
}

fn compare_token_periods(
    usage_data: &[DailyUsageData],
    range: Range,
) -> Option<(f64, String)> {
    if usage_data.is_empty() {
        return None;
    }

    let latest = match usage_data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return None,
    };

    let cutoff = range_cutoff(range, latest);
    let period_days = range.days() as i64;

    // Calculate current period total
    let current_total: u64 = usage_data
        .iter()
        .filter(|d| d.date >= cutoff)
        .map(|d| d.input_tokens + d.output_tokens)
        .sum();

    // Calculate previous period total
    let previous_cutoff = cutoff - Duration::days(period_days);
    let previous_total: u64 = usage_data
        .iter()
        .filter(|d| d.date >= previous_cutoff && d.date < cutoff)
        .map(|d| d.input_tokens + d.output_tokens)
        .sum();

    if previous_total == 0 {
        return None;
    }

    let change_pct = ((current_total as f64 - previous_total as f64) / previous_total as f64) * 100.0;
    let direction = if change_pct >= 0.0 { "↑" } else { "↓" };

    Some((change_pct, direction.to_string()))
}
