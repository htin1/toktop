use crate::app::{App, GroupBy, Provider};
use crate::models::DailyUsageData;
use crate::ui::colors::ColorPalette;
use crate::ui::content::shared;
use crate::ui::utils::format_tokens;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

struct UsageChartData {
    daily_tokens: HashMap<String, HashMap<String, (u64, u64)>>,
    item_totals: HashMap<String, (u64, u64)>,
    dates: Vec<String>,
    items: Vec<String>,
}

fn process_usage_data(data: &[DailyUsageData], group_by: GroupBy) -> UsageChartData {
    let mut daily_tokens: HashMap<String, HashMap<String, (u64, u64)>> = HashMap::new();
    let mut item_totals: HashMap<String, (u64, u64)> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();

        // Determine the grouping key based on group_by setting
        let item_key = match group_by {
            GroupBy::Model => d
                .model
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or("unknown")
                .to_string(),
            GroupBy::ApiKeys => d
                .api_key_id
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or("unknown")
                .to_string(),
        };

        let entry = daily_tokens
            .entry(date_str)
            .or_default()
            .entry(item_key.clone())
            .or_insert((0, 0));
        entry.0 += d.input_tokens;
        entry.1 += d.output_tokens;

        let total = item_totals.entry(item_key).or_insert((0, 0));
        total.0 += d.input_tokens;
        total.1 += d.output_tokens;
    }

    let mut dates: Vec<String> = daily_tokens.keys().cloned().collect();
    dates.sort();

    let mut items: Vec<String> = item_totals.keys().cloned().collect();
    items.sort();

    UsageChartData {
        daily_tokens,
        item_totals,
        dates,
        items,
    }
}

fn render_usage_legend(
    f: &mut Frame,
    area: Rect,
    items: &[String],
    item_totals: &HashMap<String, (u64, u64)>,
    item_colors: &HashMap<String, Color>,
    palette: &ColorPalette,
    group_by: GroupBy,
    api_key_names: &HashMap<String, String>,
) {
    let legend_title = match group_by {
        GroupBy::Model => "Models",
        GroupBy::ApiKeys => "API Keys",
    };

    let mut legend_lines = vec![
        Line::from(Span::styled(
            legend_title,
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for item in items {
        let color = item_colors.get(item).copied().unwrap_or(Color::White);
        let (input_total, output_total) = item_totals.get(item).copied().unwrap_or((0, 0));
        let display_item = match group_by {
            GroupBy::ApiKeys => {
                let fallback = shared::abbreviate_api_key(item);
                api_key_names.get(item).cloned().unwrap_or(fallback)
            }
            GroupBy::Model => item.clone(),
        };
        legend_lines.push(Line::from(vec![
            Span::styled(
                "   ",
                Style::default()
                    .bg(color)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(display_item),
        ]));
        legend_lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("In: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_tokens(input_total)),
            Span::raw(" "),
            Span::styled("Out: ", Style::default().fg(Color::Magenta)),
            Span::raw(format_tokens(output_total)),
        ]));
    }

    f.render_widget(
        Paragraph::new(legend_lines).alignment(Alignment::Left),
        area,
    );
}

fn render_stacked_bars(
    f: &mut Frame,
    chart_area: Rect,
    chart_data: &UsageChartData,
    item_colors: &HashMap<String, Color>,
    max_total: u64,
) {
    let total_height_per_bar = shared::BAR_HEIGHT + shared::BAR_SPACING;

    for (i, date) in chart_data.dates.iter().enumerate() {
        let y_pos = chart_area.y + (i as u16 * total_height_per_bar);

        if y_pos + shared::BAR_HEIGHT > chart_area.y + chart_area.height {
            break;
        }

        let item_tokens = &chart_data.daily_tokens[date];
        let total_tokens: u64 = item_tokens
            .values()
            .map(|(input, output)| input + output)
            .sum();

        // Date label
        let date_label_area = Rect::new(chart_area.x, y_pos, shared::DATE_LABEL_WIDTH, shared::BAR_HEIGHT);
        f.render_widget(
            Paragraph::new(date.clone()).style(Style::default().fg(Color::White)),
            date_label_area,
        );

        // Bar area
        let bar_x = chart_area.x + shared::DATE_LABEL_OFFSET;
        let bar_width = chart_area.width.saturating_sub(shared::BAR_PADDING);
        let mut current_x = bar_x;

        // Render stacked segments - one per item (model)
        for item in &chart_data.items {
            if let Some(&(input_tokens, output_tokens)) = item_tokens.get(item) {
                let total_item_tokens = input_tokens + output_tokens;

                if total_item_tokens > 0 {
                    let segment_width =
                        ((total_item_tokens as f64 / max_total as f64) * bar_width as f64) as u16;

                    if segment_width > 0 {
                        let color = item_colors.get(item).copied().unwrap_or(Color::White);
                        let segment_area = Rect::new(current_x, y_pos, segment_width, shared::BAR_HEIGHT);

                        let text = if segment_width > shared::MIN_SEGMENT_WIDTH_FOR_TOKENS {
                            format_tokens(total_item_tokens)
                        } else {
                            "".to_string()
                        };

                        shared::render_stacked_bar_segment(f, segment_area, &text, color, Color::Black);
                        current_x += segment_width;
                    }
                }
            }
        }

        // Total label
        let total_x = bar_x + bar_width + 2;
        let total_area = Rect::new(total_x, y_pos, 14, shared::BAR_HEIGHT);
        f.render_widget(
            Paragraph::new(format_tokens(total_tokens)).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            total_area,
        );
    }
}

fn render_usage_chart(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    item_colors: &HashMap<String, Color>,
) {
    let palette = ColorPalette::for_provider(provider);
    let group_by_label = match app.group_by {
        GroupBy::Model => "Model",
        GroupBy::ApiKeys => "API Keys",
    };
    let title = format!(
        "{} - Daily Token Usage by {}",
        provider.label(),
        group_by_label
    );

    let usage_data = match app.usage_data_for_provider(provider) {
        Some(data) => data,
        None => {
            shared::render_empty_state(f, area, &title, "Usage data not available");
            return;
        }
    };

    if usage_data.is_empty() {
        shared::render_empty_state(f, area, &title, "No usage data available");
        return;
    }

    let chart_data = process_usage_data(usage_data, app.group_by);

    if chart_data.dates.is_empty() {
        shared::render_empty_state(f, area, &title, "No data available");
        return;
    }

    let max_total = chart_data
        .daily_tokens
        .values()
        .map(|items| {
            items
                .values()
                .map(|(input, output)| input + output)
                .sum::<u64>()
        })
        .fold(0u64, u64::max)
        .max(1);

    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block, area);
    let inner = Block::default().borders(Borders::ALL).inner(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(shared::LEGEND_WIDTH)])
        .split(inner);

    let api_key_names = match provider {
        Provider::OpenAI => &app.data.openai_api_key_names,
        Provider::Anthropic => &app.data.anthropic_api_key_names,
    };

    render_usage_legend(
        f,
        chunks[1],
        &chart_data.items,
        &chart_data.item_totals,
        &item_colors,
        &palette,
        app.group_by,
        api_key_names,
    );

    let chart_area = chunks[0];
    render_stacked_bars(f, chart_area, &chart_data, &item_colors, max_total);
}

pub fn render_usage_view(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider);
    let group_by_label = match app.group_by {
        GroupBy::Model => "Model",
        GroupBy::ApiKeys => "API Keys",
    };
    let title = format!(
        "{} - Daily Token Usage by {}",
        provider.label(),
        group_by_label
    );

    if let Some(err) = error {
        shared::render_error_message(
            f,
            area,
            &title,
            &format!("Error loading {} Usage data: {}", provider.label(), err),
            palette.error,
        );
        return;
    }

    if !has_client {
        shared::render_empty_state(f, area, &title, "");
        return;
    }

    let usage_data = match app.usage_data_for_provider(provider) {
        Some(values) => values,
        None => {
            shared::render_empty_state(
                f,
                area,
                &title,
                &format!("{} Usage data is not wired up yet.", provider.label()),
            );
            return;
        }
    };

    if usage_data.is_empty() && app.loading {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!("Loading {} Usage data...", provider.label()),
        );
        return;
    }

    if usage_data.is_empty() {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!(
                "No {} Usage data available for the selected window.",
                provider.label()
            ),
        );
        return;
    }

    // Process usage data to get items based on group_by
    let chart_data = process_usage_data(usage_data, app.group_by);

    // Create color mapping by rotating through palette
    let item_colors = shared::create_color_mapping(&chart_data.items, palette);

    render_usage_chart(f, app, area, provider, &item_colors);
}

