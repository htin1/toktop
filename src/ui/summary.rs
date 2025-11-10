use crate::app::{App, Range};
use crate::models::{DailyData, DailyUsageData};
use crate::ui::banner;
use crate::ui::colors::ColorPalette;
use crate::ui::utils::format_tokens;
use chrono::{DateTime, Duration, Utc};
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
    let total_requests = calculate_total_requests(&info.usage_data, app.range);

    // Calculate period comparisons
    let cost_period_comparison =
        compare_periods(&info.cost_data, app.range, |d| d.date, |d| d.cost);
    let token_period_comparison = compare_periods(
        &info.usage_data,
        app.range,
        |d| d.date,
        |d| (d.input_tokens + d.output_tokens) as f64,
    );

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

    if app.range == crate::app::Range::SevenDays {
        add_period_comparison(&mut left_text, cost_period_comparison);
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

        if app.range == crate::app::Range::SevenDays {
            add_period_comparison(&mut left_text, token_period_comparison);
        }
    }

    // Right column: Trends & Info
    let mut right_text = vec![];

    // Total requests (OpenAI only)
    if let Some(requests) = total_requests {
        right_text.push(Line::from(""));
        right_text.push(Line::from(vec![
            Span::styled(
                format!("Total Requests ({}): ", app.range.label()),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!("{}", requests),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        let avg_requests_per_day = requests as f64 / range_days;
        right_text.push(Line::from(vec![
            Span::styled("Average per day: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}", avg_requests_per_day),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

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

fn add_period_comparison(text: &mut Vec<Line>, comparison: Option<(f64, String)>) {
    if let Some((change_pct, direction)) = comparison {
        let change_color = if change_pct >= 0.0 {
            Color::Red
        } else {
            Color::Green
        };
        text.push(Line::from(vec![
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

fn range_cutoff(range: Range, latest: DateTime<Utc>) -> DateTime<Utc> {
    let span = range.days().saturating_sub(1);
    latest - Duration::days(span)
}

fn summarize_cost(
    data: &[DailyData],
    range: Range,
) -> (f64, Option<(DateTime<Utc>, DateTime<Utc>)>) {
    if data.is_empty() {
        return (0.0, None);
    }

    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return (0.0, None),
    };
    let cutoff = range_cutoff(range, latest);
    let filtered: Vec<_> = data.iter().filter(|d| d.date >= cutoff).collect();

    if filtered.is_empty() {
        return (0.0, None);
    }

    let total: f64 = filtered.iter().map(|d| d.cost).sum();
    let min_date = filtered.iter().map(|d| d.date).min().unwrap();
    let max_date = filtered.iter().map(|d| d.date).max().unwrap();

    (total, Some((min_date, max_date)))
}

fn summarize_usage(
    data: &[DailyUsageData],
    range: Range,
) -> ((u64, u64), Option<(DateTime<Utc>, DateTime<Utc>)>) {
    if data.is_empty() {
        return ((0, 0), None);
    }

    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return ((0, 0), None),
    };
    let cutoff = range_cutoff(range, latest);
    let filtered: Vec<_> = data.iter().filter(|d| d.date >= cutoff).collect();

    if filtered.is_empty() {
        return ((0, 0), None);
    }

    let input_total: u64 = filtered.iter().map(|d| d.input_tokens).sum();
    let output_total: u64 = filtered.iter().map(|d| d.output_tokens).sum();
    let min_date = filtered.iter().map(|d| d.date).min().unwrap();
    let max_date = filtered.iter().map(|d| d.date).max().unwrap();

    ((input_total, output_total), Some((min_date, max_date)))
}

fn calculate_cost_per_million_tokens(total_cost: f64, total_tokens: u64) -> Option<f64> {
    if total_tokens == 0 {
        return None;
    }
    let tokens_in_millions = total_tokens as f64 / 1_000_000.0;
    Some(total_cost / tokens_in_millions)
}

fn calculate_cache_hit_rate(usage_data: &[DailyUsageData], range: Range) -> Option<f64> {
    let latest = usage_data.iter().map(|d| d.date).max()?;
    let cutoff = range_cutoff(range, latest);

    let (cache_read_total, uncached_total): (u64, u64) = usage_data
        .iter()
        .filter(|d| d.date >= cutoff)
        .filter_map(|d| Some((d.cache_read_input_tokens?, d.uncached_input_tokens?)))
        .fold((0, 0), |(a, b), (c, u)| (a + c, b + u));

    let total_cacheable = cache_read_total + uncached_total;
    if total_cacheable == 0 {
        return None;
    }

    Some((cache_read_total as f64 / total_cacheable as f64) * 100.0)
}

fn calculate_total_requests(usage_data: &[DailyUsageData], range: Range) -> Option<u64> {
    let latest = usage_data.iter().map(|d| d.date).max()?;
    let cutoff = range_cutoff(range, latest);

    let total: u64 = usage_data
        .iter()
        .filter(|d| d.date >= cutoff)
        .filter_map(|d| d.num_requests)
        .sum();

    if total > 0 {
        Some(total)
    } else {
        None
    }
}

fn compare_periods<T>(
    data: &[T],
    range: Range,
    extract_date: impl Fn(&T) -> DateTime<Utc>,
    extract_value: impl Fn(&T) -> f64,
) -> Option<(f64, String)> {
    if data.is_empty() {
        return None;
    }

    let latest = data.iter().map(&extract_date).max()?;
    let cutoff = range_cutoff(range, latest);
    let period_days = range.days() as i64;
    let previous_cutoff = cutoff - Duration::days(period_days);

    let current: f64 = data
        .iter()
        .filter(|d| extract_date(d) >= cutoff)
        .map(&extract_value)
        .sum();
    let previous: f64 = data
        .iter()
        .filter(|d| {
            let date = extract_date(d);
            date >= previous_cutoff && date < cutoff
        })
        .map(&extract_value)
        .sum();

    if previous == 0.0 {
        return None;
    }

    let change_pct = ((current - previous) / previous) * 100.0;
    Some((
        change_pct,
        if change_pct >= 0.0 { "↑" } else { "↓" }.to_string(),
    ))
}
