use crate::app::{App, GroupBy, Range, View};
use crate::models::DailyUsageData;
use crate::provider::Provider;
use crate::ui::colors::ColorPalette;
use crate::ui::content::shared;
use crate::ui::utils::format_tokens;
use chrono::Duration;
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

fn filter_usage_data_by_range(data: &[DailyUsageData], range: Range) -> Vec<DailyUsageData> {
    let latest_date = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return Vec::new(),
    };
    let span = range.days().saturating_sub(1);
    let cutoff = latest_date - Duration::days(span);
    data.iter().filter(|d| d.date >= cutoff).cloned().collect()
}

fn apply_item_filter(
    data: &[DailyUsageData],
    group_by: GroupBy,
    selected_filter: Option<&String>,
) -> Vec<DailyUsageData> {
    if let Some(filter) = selected_filter {
        data.iter()
            .filter(|d| match group_by {
                GroupBy::Model => d
                    .model
                    .as_ref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false),
                GroupBy::ApiKeys => d
                    .api_key_id
                    .as_ref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false),
            })
            .cloned()
            .collect()
    } else {
        data.to_vec()
    }
}

fn process_usage_data(data: &[DailyUsageData], group_by: GroupBy) -> UsageChartData {
    let mut daily_tokens: HashMap<String, HashMap<String, (u64, u64)>> = HashMap::new();
    let mut item_totals: HashMap<String, (u64, u64)> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();

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

fn render_vertical_stacked_bars(
    f: &mut Frame,
    chart_area: Rect,
    chart_data: &UsageChartData,
    item_colors: &HashMap<String, Color>,
    max_total: u64,
) -> bool {
    if chart_area.width == 0 || chart_area.height <= 1 || max_total == 0 {
        return false;
    }

    let label_height: u16 = 1;
    let value_label_height: u16 = 1;
    let bar_area_height = chart_area
        .height
        .saturating_sub(label_height)
        .saturating_sub(value_label_height)
        .saturating_sub(1);
    if bar_area_height == 0 {
        return false;
    }
    let bars_y = chart_area.y + value_label_height;

    let layout = match shared::vertical_bar_layout(chart_data.dates.len(), chart_area.width) {
        Some(layout) => layout,
        None => return false,
    };

    let end_index = layout.start_index + layout.visible_bars;

    for (visible_idx, date_idx) in (layout.start_index..end_index).enumerate() {
        let date = &chart_data.dates[date_idx];
        let item_tokens = match chart_data.daily_tokens.get(date) {
            Some(values) => values,
            None => continue,
        };
        let total_tokens: u64 = item_tokens
            .values()
            .map(|(input, output)| input + output)
            .sum();
        let bar_x = chart_area.x
            + layout.offset
            + (visible_idx as u16) * (layout.bar_width + layout.spacing);

        let mut used_height = 0;
        let mut top_segment_area: Option<Rect> = None;
        for item in &chart_data.items {
            if let Some(&(input_tokens, output_tokens)) = item_tokens.get(item) {
                let total_item_tokens = input_tokens + output_tokens;
                if total_item_tokens == 0 {
                    continue;
                }

                let mut segment_height = ((total_item_tokens as f64 / max_total as f64)
                    * bar_area_height as f64)
                    .round() as u16;
                if segment_height == 0 {
                    segment_height = 1;
                }
                let remaining = bar_area_height.saturating_sub(used_height);
                if remaining == 0 {
                    break;
                }
                if segment_height > remaining {
                    segment_height = remaining;
                }

                let segment_y = bars_y + bar_area_height - used_height - segment_height;
                let color = item_colors.get(item).copied().unwrap_or(Color::White);
                let segment_area = Rect::new(bar_x, segment_y, layout.bar_width, segment_height);
                shared::render_stacked_bar_segment(f, segment_area, "", color, Color::Black);
                top_segment_area = Some(segment_area);
                used_height += segment_height;
            }
        }

        if used_height == 0 && bar_area_height > 0 {
            let marker_y = bars_y + bar_area_height - 1;
            shared::render_stacked_bar_segment(
                f,
                Rect::new(bar_x, marker_y, layout.bar_width, 1),
                "",
                Color::DarkGray,
                Color::Black,
            );
        }

        if total_tokens > 0 {
            if let Some(segment_area) = top_segment_area {
                let label_y = segment_area.y - 1;
                f.render_widget(
                    Paragraph::new(format_tokens(total_tokens))
                        .alignment(Alignment::Center)
                        .style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    Rect::new(bar_x, label_y, layout.bar_width, 1),
                );
            }
        }

        let label_area = Rect::new(
            bar_x,
            bars_y + bar_area_height,
            layout.bar_width,
            label_height,
        );
        let label_text = shared::compact_date_label(date, layout.bar_width);
        f.render_widget(
            Paragraph::new(label_text).alignment(Alignment::Center),
            label_area,
        );
    }

    true
}

fn render_usage_chart(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    item_colors: &HashMap<String, Color>,
    chart_data: &UsageChartData,
    title: &str,
) {
    let palette = ColorPalette::for_provider(provider);

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
    if !render_vertical_stacked_bars(f, chart_area, chart_data, &item_colors, max_total) {
        shared::render_empty_state(
            f,
            chart_area,
            "Chart",
            "Not enough space to render usage chart",
        );
    }
}

pub fn render_usage_view(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider, View::Usage);
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
                            .unwrap_or_else(|| shared::abbreviate_api_key(filter))
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

    let range_filtered_data = filter_usage_data_by_range(usage_data, app.range);
    let all_items_chart_data = process_usage_data(&range_filtered_data, app.group_by);
    let all_item_colors = shared::create_color_mapping(&all_items_chart_data.items, palette);
    
    let filtered_data = apply_item_filter(
        &range_filtered_data,
        app.group_by,
        app.selected_filter.as_ref(),
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
    let item_colors: HashMap<String, Color> = chart_data.items
        .iter()
        .filter_map(|item| {
            all_item_colors.get(item).map(|color| (item.clone(), *color))
        })
        .collect();

    render_usage_chart(f, app, area, provider, &item_colors, &chart_data, &title);
}
