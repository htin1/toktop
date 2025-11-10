use crate::app::{App, GroupBy, Range, View};
use crate::models::{DailyData, DailyUsageData};
use crate::ui::banner;
use crate::ui::colors::ColorPalette;
use crate::ui::content::shared;
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
    let has_data = !info.cost_data.is_empty() || !info.usage_data.is_empty();

    if app.loading || !has_data {
        let mut text = vec![];
        text.extend(banner::render_animated_banner(app, &palette));
        f.render_widget(
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Summary")),
            area,
        );
        return;
    }

    let cost_filter = if app.current_view == View::Cost {
        app.selected_filter.as_ref()
    } else {
        None
    };
    let (total_cost, cost_bounds) =
        summarize_cost(&info.cost_data, app.range, cost_filter, GroupBy::Model);
    let cost_period_comparison = compare_periods(
        &info.cost_data,
        app.range,
        |d| d.date,
        |d| d.cost,
        cost_filter,
        |d| shared::extract_trimmed_string(&d.line_item),
    );

    let usage_filter = if app.current_view == View::Usage {
        app.selected_filter.as_ref()
    } else {
        None
    };
    let ((input_tokens, output_tokens), usage_bounds) =
        summarize_usage(&info.usage_data, app.range, usage_filter, app.group_by);
    let cache_hit_rate =
        calculate_cache_hit_rate(&info.usage_data, app.range, usage_filter, app.group_by);
    let total_requests =
        calculate_total_requests(&info.usage_data, app.range, usage_filter, app.group_by);
    let token_period_comparison = compare_periods(
        &info.usage_data,
        app.range,
        |d| d.date,
        |d| (d.input_tokens + d.output_tokens) as f64,
        usage_filter,
        |d| match app.group_by {
            GroupBy::Model => shared::extract_trimmed_string(&d.model),
            GroupBy::ApiKeys => shared::extract_trimmed_string(&d.api_key_id),
        },
    );

    let range_days = app.range.days().max(1) as f64;
    let avg_cost_per_day = total_cost / range_days;
    let total_tokens = input_tokens + output_tokens;
    let avg_tokens_per_day = total_tokens as f64 / range_days;

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

    let block = Block::default().borders(Borders::ALL).title("Summary");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Vertical layout: main content area + date range footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    // Horizontal layout: Cost column (left) + Usage column (right)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    // Build Cost column content
    let mut cost_text = vec![];
    let cost_header = if let Some(filter) = cost_filter {
        format!("Cost: {}", filter)
    } else {
        "Cost: All".to_string()
    };
    cost_text.push(Line::from(Span::styled(
        cost_header,
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD),
    )));
    cost_text.push(Line::from(""));
    add_labeled_value(
        &mut cost_text,
        format!("Total ({}): ", app.range.label()),
        format!("${:.2}", total_cost),
        &palette,
    );
    add_labeled_value(
        &mut cost_text,
        "Average per day: ",
        format!("${:.2}", avg_cost_per_day),
        &palette,
    );
    if app.range == crate::app::Range::SevenDays {
        add_period_comparison(&mut cost_text, cost_period_comparison);
    }

    // Build Usage column content
    let mut usage_text = vec![];
    let usage_header = if let Some(filter) = usage_filter {
        let display_name = match app.group_by {
            GroupBy::Model => filter.clone(),
            GroupBy::ApiKeys => {
                let api_key_names = &info.api_key_names;
                api_key_names
                    .get(filter)
                    .cloned()
                    .unwrap_or_else(|| shared::abbreviate_api_key(filter))
            }
        };
        format!("Usage: {}", display_name)
    } else {
        "Usage: All".to_string()
    };
    usage_text.push(Line::from(Span::styled(
        usage_header,
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD),
    )));
    usage_text.push(Line::from(""));
    add_labeled_value(
        &mut usage_text,
        format!("Total Tokens ({}): ", app.range.label()),
        format_tokens(total_tokens),
        &palette,
    );
    add_labeled_value(
        &mut usage_text,
        "Average per day: ",
        format_tokens(avg_tokens_per_day as u64),
        &palette,
    );
    usage_text.push(Line::from(vec![
        Span::styled("Input: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format_tokens(input_tokens),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Output: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format_tokens(output_tokens),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    if app.range == crate::app::Range::SevenDays {
        add_period_comparison(&mut usage_text, token_period_comparison);
    }
    if let Some(requests) = total_requests {
        usage_text.push(Line::from(""));
        add_labeled_value(
            &mut usage_text,
            format!("Total Requests ({}): ", app.range.label()),
            format!("{}", requests),
            &palette,
        );
        let avg_requests_per_day = requests as f64 / range_days;
        add_labeled_value(
            &mut usage_text,
            "Average per day: ",
            format!("{:.0}", avg_requests_per_day),
            &palette,
        );
    }
    if let Some(hit_rate) = cache_hit_rate {
        usage_text.push(Line::from(""));
        add_labeled_value(
            &mut usage_text,
            "Cache hit rate: ",
            format!("{:.1}%", hit_rate),
            &palette,
        );
    }

    // Date range footer
    let date_range_text = vec![Line::from(vec![
        Span::styled("Date Range: ", Style::default().fg(palette.primary)),
        Span::raw(date_range),
    ])];

    // Render columns and footer
    f.render_widget(Paragraph::new(cost_text), columns[0]);
    f.render_widget(Paragraph::new(usage_text), columns[1]);
    f.render_widget(Paragraph::new(date_range_text), main_layout[1]);
}

fn add_labeled_value(
    text: &mut Vec<Line>,
    label: impl Into<String>,
    value: impl Into<String>,
    palette: &ColorPalette,
) {
    text.push(Line::from(vec![
        Span::styled(label.into(), Style::default().fg(Color::Gray)),
        Span::styled(
            value.into(),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
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
    selected_filter: Option<&String>,
    group_by: GroupBy,
) -> (f64, Option<(DateTime<Utc>, DateTime<Utc>)>) {
    if data.is_empty() {
        return (0.0, None);
    }

    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return (0.0, None),
    };
    let cutoff = range_cutoff(range, latest);
    let mut filtered: Vec<_> = data.iter().filter(|d| d.date >= cutoff).collect();

    if let Some(filter) = selected_filter {
        if group_by == GroupBy::Model {
            filtered = filtered
                .into_iter()
                .filter(|d| {
                    shared::extract_trimmed_string(&d.line_item)
                        .map(|s| s == filter.as_str())
                        .unwrap_or(false)
                })
                .collect();
        }
    }

    if filtered.is_empty() {
        return (0.0, None);
    }

    let total: f64 = filtered.iter().map(|d| d.cost).sum();
    let min_date = filtered.iter().map(|d| d.date).min().unwrap();
    let max_date = filtered.iter().map(|d| d.date).max().unwrap();

    (total, Some((min_date, max_date)))
}

fn filter_usage_data_by_range_and_filter<'a>(
    data: &'a [DailyUsageData],
    range: Range,
    selected_filter: Option<&String>,
    group_by: GroupBy,
) -> Vec<&'a DailyUsageData> {
    let latest = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return Vec::new(),
    };
    let cutoff = range_cutoff(range, latest);
    let mut filtered: Vec<_> = data.iter().filter(|d| d.date >= cutoff).collect();

    if let Some(filter) = selected_filter {
        filtered = filtered
            .into_iter()
            .filter(|d| {
                let field_value = match group_by {
                    GroupBy::Model => shared::extract_trimmed_string(&d.model),
                    GroupBy::ApiKeys => shared::extract_trimmed_string(&d.api_key_id),
                };
                field_value.map(|s| s == filter.as_str()).unwrap_or(false)
            })
            .collect();
    }

    filtered
}

fn summarize_usage(
    data: &[DailyUsageData],
    range: Range,
    selected_filter: Option<&String>,
    group_by: GroupBy,
) -> ((u64, u64), Option<(DateTime<Utc>, DateTime<Utc>)>) {
    let filtered = filter_usage_data_by_range_and_filter(data, range, selected_filter, group_by);

    if filtered.is_empty() {
        return ((0, 0), None);
    }

    let input_total: u64 = filtered.iter().map(|d| d.input_tokens).sum();
    let output_total: u64 = filtered.iter().map(|d| d.output_tokens).sum();
    let min_date = filtered.iter().map(|d| d.date).min().unwrap();
    let max_date = filtered.iter().map(|d| d.date).max().unwrap();

    ((input_total, output_total), Some((min_date, max_date)))
}

fn calculate_cache_hit_rate(
    usage_data: &[DailyUsageData],
    range: Range,
    selected_filter: Option<&String>,
    group_by: GroupBy,
) -> Option<f64> {
    let filtered =
        filter_usage_data_by_range_and_filter(usage_data, range, selected_filter, group_by);

    let (cache_read_total, uncached_total): (u64, u64) = filtered
        .iter()
        .filter_map(|d| Some((d.cache_read_input_tokens?, d.uncached_input_tokens?)))
        .fold((0, 0), |(a, b), (c, u)| (a + c, b + u));

    let total_cacheable = cache_read_total + uncached_total;
    if total_cacheable == 0 {
        return None;
    }

    Some((cache_read_total as f64 / total_cacheable as f64) * 100.0)
}

fn calculate_total_requests(
    usage_data: &[DailyUsageData],
    range: Range,
    selected_filter: Option<&String>,
    group_by: GroupBy,
) -> Option<u64> {
    let filtered =
        filter_usage_data_by_range_and_filter(usage_data, range, selected_filter, group_by);
    let total: u64 = filtered.iter().filter_map(|d| d.num_requests).sum();

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
    selected_filter: Option<&String>,
    extract_filter_field: impl Fn(&T) -> Option<&str>,
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
        .filter(|d| {
            if let Some(filter) = selected_filter {
                extract_filter_field(d)
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .map(&extract_value)
        .sum();
    let previous: f64 = data
        .iter()
        .filter(|d| {
            let date = extract_date(d);
            date >= previous_cutoff && date < cutoff
        })
        .filter(|d| {
            if let Some(filter) = selected_filter {
                extract_filter_field(d)
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false)
            } else {
                true
            }
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
