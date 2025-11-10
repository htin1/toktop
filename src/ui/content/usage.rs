use crate::app::{App, GroupBy, View};
use crate::models::DailyUsageData;
use crate::provider::Provider;
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

        let item_key = match group_by {
            GroupBy::Model => shared::extract_trimmed_string(&d.model)
                .unwrap_or("unknown")
                .to_string(),
            GroupBy::ApiKeys => shared::extract_trimmed_string(&d.api_key_id)
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
            Span::styled(
                format_tokens(input_total),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("Out: ", Style::default().fg(Color::Magenta)),
            Span::styled(
                format_tokens(output_total),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(legend_lines).alignment(Alignment::Left),
        area,
    );
}

fn render_usage_chart(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    provider: Provider,
    item_colors: &HashMap<String, Color>,
    chart_data: &UsageChartData,
    title: &str,
    scroll_offset: usize,
) -> Option<usize> {
    let palette = ColorPalette::for_provider(provider);

    if chart_data.dates.is_empty() {
        app.chart_scrollbar_visible = false;
        shared::render_empty_state(f, area, &title, "No data available");
        return None;
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
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(shared::LEGEND_WIDTH)])
        .split(inner);

    let api_key_names = &app.provider_info(provider).api_key_names;

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
    match shared::render_vertical_stacked_bars(
        f,
        chart_area,
        &chart_data.dates,
        &chart_data.items,
        |date, item| {
            chart_data
                .daily_tokens
                .get(date)
                .and_then(|items| items.get(item))
                .map(|(input, output)| (*input + *output) as f64)
        },
        |date| {
            chart_data
                .daily_tokens
                .get(date)
                .map(|items| {
                    items
                        .values()
                        .map(|(input, output)| (*input + *output) as f64)
                        .sum()
                })
                .unwrap_or(0.0)
        },
        |total| format_tokens(total as u64),
        |value| format_tokens(value as u64),
        &item_colors,
        max_total as f64,
        scroll_offset,
        app.show_segment_values,
    ) {
        Some(layout) => {
            shared::handle_chart_scrollbar(
                f,
                app,
                chart_area,
                chart_data.dates.len(),
                layout,
                palette.accent,
            );
            Some(layout.start_index)
        }
        None => {
            app.chart_scrollbar_visible = false;
            shared::render_empty_state(
                f,
                chart_area,
                "Chart",
                "Not enough space to render usage chart",
            );
            None
        }
    }
}

pub fn render_usage_view(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider, View::Usage).cloned();
    let group_by_label = match app.group_by {
        GroupBy::Model => "Model",
        GroupBy::ApiKeys => "API Keys",
    };
    let filter_suffix = if let Some(ref filter) = app.selected_filter {
        let display_name = match app.group_by {
            GroupBy::Model => filter.clone(),
            GroupBy::ApiKeys => {
                let api_key_names = &app.provider_info(provider).api_key_names;
                api_key_names
                    .get(filter)
                    .cloned()
                    .unwrap_or_else(|| "Unknown Key".to_string())
            }
        };
        format!(" - {}", display_name)
    } else {
        String::new()
    };
    let title = format!(
        "{} - Daily Token Usage by {}{}",
        provider.label(),
        group_by_label,
        filter_suffix
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

    let range_filtered_data = {
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
        app.filter_usage_data_by_range(usage_data)
    };

    if range_filtered_data.is_empty() {
        let msg = if app.loading {
            format!("Loading {} Usage data...", provider.label())
        } else {
            format!(
                "No {} Usage data available for the selected window.",
                provider.label()
            )
        };
        shared::render_empty_state(f, area, &title, &msg);
        return;
    }

    let all_items_chart_data = process_usage_data(&range_filtered_data, app.group_by);
    let all_item_colors = shared::create_color_mapping(&all_items_chart_data.items, palette);

    let filtered_data = shared::apply_filter(
        &range_filtered_data,
        app.selected_filter.as_ref(),
        |d| match app.group_by {
            GroupBy::Model => shared::extract_trimmed_string(&d.model),
            GroupBy::ApiKeys => shared::extract_trimmed_string(&d.api_key_id),
        },
    );

    if filtered_data.is_empty() {
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

    let chart_data = process_usage_data(&filtered_data, app.group_by);
    let item_colors = shared::filter_item_colors(&all_item_colors, &chart_data.items);

    let scroll_offset = {
        let info = app.provider_info(provider);
        info.usage_chart_scroll
    };

    if let Some(actual_scroll) = render_usage_chart(
        f,
        app,
        area,
        provider,
        &item_colors,
        &chart_data,
        &title,
        scroll_offset,
    ) {
        let info = app.provider_info_mut(provider);
        info.usage_chart_scroll = actual_scroll;
    }
}
