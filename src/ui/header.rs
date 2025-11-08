use crate::app::{App, Range};
use crate::models::{DailyData, DailyUsageData};
use crate::ui::banner;
use crate::ui::colors::ColorPalette;
use crate::ui::utils::format_tokens;
use chrono::{Duration, Utc};
use ratatui::{
    layout::Rect,
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
    let avg_per_day = total_cost / range_days;

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

    let mut text = vec![];

    text.push(Line::from(""));
    text.push(Line::from(vec![
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
    text.push(Line::from(vec![
        Span::styled("Average per day: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${:.2}", avg_per_day),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    if input_tokens > 0 || output_tokens > 0 {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled(
                format!("Total Tokens ({}): ", app.range.label()),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format_tokens(input_tokens + output_tokens),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Input: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_tokens(input_tokens)),
            Span::raw(" | "),
            Span::styled("Output: ", Style::default().fg(Color::Magenta)),
            Span::raw(format_tokens(output_tokens)),
        ]));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("Date Range: ", Style::default().fg(Color::Gray)),
        Span::raw(date_range),
    ]));

    f.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Summary")),
        area,
    );
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
