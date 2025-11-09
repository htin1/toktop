use crate::app::{App, Range, View};
use crate::models::DailyData;
use crate::provider::Provider;
use crate::ui::colors::ColorPalette;
use crate::ui::content::shared;
use chrono::Duration;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

struct CostChartData {
    daily_costs: HashMap<String, HashMap<String, f64>>,
    item_totals: HashMap<String, f64>,
    dates: Vec<String>,
    items: Vec<String>,
}

fn filter_cost_data_by_range(data: &[DailyData], range: Range) -> Vec<DailyData> {
    let latest_date = match data.iter().map(|d| d.date).max() {
        Some(date) => date,
        None => return Vec::new(),
    };
    let span = range.days().saturating_sub(1);
    let cutoff = latest_date - Duration::days(span);
    data.iter().filter(|d| d.date >= cutoff).cloned().collect()
}

fn apply_model_filter(data: &[DailyData], selected_filter: Option<&String>) -> Vec<DailyData> {
    if let Some(filter) = selected_filter {
        data.iter()
            .filter(|d| {
                d.line_item
                    .as_ref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    } else {
        data.to_vec()
    }
}

fn process_cost_data(data: &[DailyData]) -> CostChartData {
    let mut daily_costs: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut item_totals: HashMap<String, f64> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();
        let line_item = d
            .line_item
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_string();

        *daily_costs
            .entry(date_str)
            .or_default()
            .entry(line_item.clone())
            .or_insert(0.0) += d.cost;

        *item_totals.entry(line_item).or_insert(0.0) += d.cost;
    }

    let mut dates: Vec<String> = daily_costs.keys().cloned().collect();
    dates.sort();

    let mut items: Vec<String> = item_totals.keys().cloned().collect();
    items.sort();

    CostChartData {
        daily_costs,
        item_totals,
        dates,
        items,
    }
}

fn render_cost_legend(
    f: &mut Frame,
    area: Rect,
    items: &[String],
    item_totals: &HashMap<String, f64>,
    item_colors: &HashMap<String, Color>,
    palette: &ColorPalette,
) {
    let mut legend_items: Vec<String> = items
        .iter()
        .filter(|item| item_totals.get(*item).copied().unwrap_or(0.0) >= shared::COST_THRESHOLD)
        .cloned()
        .collect();

    if legend_items.is_empty() {
        legend_items = items.to_vec();
    }

    let mut legend_lines = vec![
        Line::from(Span::styled(
            "Models (>$1)",
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for item in &legend_items {
        let color = item_colors.get(item).copied().unwrap_or(Color::White);
        let cost = item_totals.get(item).copied().unwrap_or(0.0);
        let cost_str = if cost >= shared::COST_THRESHOLD {
            format!("${:.2}", cost)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            format!("${:.2}", cost)
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
            Span::raw(item.clone()),
        ]));
        legend_lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("Cost: ", Style::default().fg(palette.primary)),
            Span::styled(
                cost_str,
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

fn render_cost_chart(
    f: &mut Frame,
    app: &mut App,
    data: &[DailyData],
    area: Rect,
    title: &str,
    provider: Provider,
    item_colors: &HashMap<String, Color>,
    scroll_offset: usize,
) -> Option<usize> {
    let palette = ColorPalette::for_provider(provider);
    let chart_data = process_cost_data(data);

    if chart_data.dates.is_empty() {
        app.chart_scrollbar_visible = false;
        shared::render_empty_state(f, area, title, "No data available");
        return None;
    }

    let max_total = chart_data
        .daily_costs
        .values()
        .map(|models| models.values().sum::<f64>())
        .fold(0.0, f64::max)
        .max(1.0);

    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block, area);
    let inner = Block::default().borders(Borders::ALL).inner(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(shared::LEGEND_WIDTH)])
        .split(inner);

    render_cost_legend(
        f,
        chunks[1],
        &chart_data.items,
        &chart_data.item_totals,
        &item_colors,
        &palette,
    );

    let chart_area = chunks[0];
    match render_vertical_cost_bars(
        f,
        chart_area,
        &chart_data,
        item_colors,
        max_total,
        scroll_offset,
    ) {
        Some(layout) => {
            let scrollbar_visible = chart_data.dates.len() > layout.visible_bars
                && chart_area.height >= shared::HORIZONTAL_SCROLLBAR_HEIGHT;
            app.chart_scrollbar_visible = scrollbar_visible;

            if scrollbar_visible {
                let scrollbar_height = shared::HORIZONTAL_SCROLLBAR_HEIGHT.min(chart_area.height);
                let scrollbar_area = Rect::new(
                    chart_area.x,
                    chart_area.y + chart_area.height.saturating_sub(scrollbar_height),
                    chart_area.width,
                    scrollbar_height,
                );
                shared::render_horizontal_scrollbar(
                    f,
                    scrollbar_area,
                    chart_data.dates.len(),
                    layout.visible_bars,
                    layout.start_index,
                    palette.accent,
                );
            }
            Some(layout.start_index)
        }
        None => {
            app.chart_scrollbar_visible = false;
            shared::render_empty_state(
                f,
                chart_area,
                "Chart",
                "Not enough space to render cost chart",
            );
            None
        }
    }
}

fn render_vertical_cost_bars(
    f: &mut Frame,
    chart_area: Rect,
    chart_data: &CostChartData,
    item_colors: &HashMap<String, Color>,
    max_total: f64,
    scroll_offset: usize,
) -> Option<shared::VerticalBarLayout> {
    if chart_area.width == 0 || chart_area.height <= 1 || max_total <= 0.0 {
        return None;
    }

    let label_height: u16 = 1;
    let value_label_height: u16 = 1;
    let scrollbar_height = shared::HORIZONTAL_SCROLLBAR_HEIGHT;
    let bar_area_height = chart_area
        .height
        .saturating_sub(label_height)
        .saturating_sub(value_label_height)
        .saturating_sub(scrollbar_height);
    if bar_area_height == 0 {
        return None;
    }
    let bars_y = chart_area.y + value_label_height;

    let layout = match shared::vertical_bar_layout(
        chart_data.dates.len(),
        chart_area.width,
        scroll_offset,
    ) {
        Some(layout) => layout,
        None => return None,
    };

    let end_index = layout.start_index + layout.visible_bars;

    for (visible_idx, date_idx) in (layout.start_index..end_index).enumerate() {
        let date = &chart_data.dates[date_idx];
        let model_costs = match chart_data.daily_costs.get(date) {
            Some(values) => values,
            None => continue,
        };

        let total_cost: f64 = model_costs.values().sum();
        let bar_x = chart_area.x
            + layout.offset
            + (visible_idx as u16) * (layout.bar_width + layout.spacing);

        let mut used_height = 0;
        let mut top_segment_area: Option<Rect> = None;
        for item in &chart_data.items {
            if let Some(&cost) = model_costs.get(item) {
                if cost <= 0.0 {
                    continue;
                }

                let mut segment_height =
                    ((cost / max_total) * bar_area_height as f64).round() as u16;
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

        if total_cost > 0.0 {
            if let Some(segment_area) = top_segment_area {
                let label_y = segment_area.y - 1;
                f.render_widget(
                    Paragraph::new(format!("${:.0}", total_cost))
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

    Some(layout)
}

pub fn render_cost_view(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider, View::Cost).cloned();
    let filter_suffix = if let Some(ref filter) = app.selected_filter {
        format!(" - {}", filter)
    } else {
        String::new()
    };
    let title = format!(
        "{} - Daily Cost by Model{}",
        provider.label(),
        filter_suffix
    );

    if let Some(err) = error {
        shared::render_error_message(
            f,
            area,
            &title,
            &format!("Error loading {} Cost data: {}", provider.label(), err),
            palette.error,
        );
        return;
    }

    if !has_client {
        shared::render_empty_state(f, area, &title, "");
        return;
    }

    let range_filtered_data = {
        let data = match app.data_for_provider(provider) {
            Some(values) => values,
            None => {
                shared::render_empty_state(
                    f,
                    area,
                    &title,
                    &format!("{} Cost data is not wired up yet.", provider.label()),
                );
                return;
            }
        };
        filter_cost_data_by_range(data, app.range)
    };

    if range_filtered_data.is_empty() && app.loading {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!("Loading {} Cost data...", provider.label()),
        );
        return;
    }

    if range_filtered_data.is_empty() {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!(
                "No {} Cost data available for the selected window.",
                provider.label()
            ),
        );
        return;
    }

    let all_items_chart_data = process_cost_data(&range_filtered_data);
    let all_item_colors = shared::create_color_mapping(&all_items_chart_data.items, palette);

    let filtered_data = apply_model_filter(&range_filtered_data, app.selected_filter.as_ref());

    if filtered_data.is_empty() {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!(
                "No {} Cost data available for the selected window.",
                provider.label()
            ),
        );
        return;
    }

    let chart_data = process_cost_data(&filtered_data);
    let item_colors: HashMap<String, Color> = chart_data
        .items
        .iter()
        .filter_map(|item| {
            all_item_colors
                .get(item)
                .map(|color| (item.clone(), *color))
        })
        .collect();

    let scroll_offset = {
        let info = app.provider_info(provider);
        info.cost_chart_scroll
    };

    if let Some(actual_scroll) = render_cost_chart(
        f,
        app,
        &filtered_data,
        area,
        &title,
        provider,
        &item_colors,
        scroll_offset,
    ) {
        let info = app.provider_info_mut(provider);
        info.cost_chart_scroll = actual_scroll;
    }
}
