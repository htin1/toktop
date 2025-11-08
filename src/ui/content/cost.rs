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
        .filter(|item| {
            item_totals.get(*item).copied().unwrap_or(0.0) > shared::COST_THRESHOLD_FOR_LEGEND
        })
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
    }

    f.render_widget(
        Paragraph::new(legend_lines).alignment(Alignment::Left),
        area,
    );
}

fn render_cost_chart(
    f: &mut Frame,
    data: &[DailyData],
    area: Rect,
    title: &str,
    provider: Provider,
    item_colors: &HashMap<String, Color>,
) {
    let palette = ColorPalette::for_provider(provider);
    let chart_data = process_cost_data(data);

    if chart_data.dates.is_empty() {
        shared::render_empty_state(f, area, title, "No data available");
        return;
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
    if !render_vertical_cost_bars(f, chart_area, &chart_data, item_colors, max_total) {
        shared::render_empty_state(
            f,
            chart_area,
            "Chart",
            "Not enough space to render cost chart",
        );
    }
}

fn render_vertical_cost_bars(
    f: &mut Frame,
    chart_area: Rect,
    chart_data: &CostChartData,
    item_colors: &HashMap<String, Color>,
    max_total: f64,
) -> bool {
    if chart_area.width == 0 || chart_area.height <= 1 || max_total <= 0.0 {
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

    true
}

pub fn render_cost_view(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider, View::Cost);
    let title = format!("{} - Daily Cost by Model", provider.label());

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

    if data.is_empty() && app.loading {
        shared::render_empty_state(
            f,
            area,
            &title,
            &format!("Loading {} Cost data...", provider.label()),
        );
        return;
    }

    if data.is_empty() {
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

    let filtered_data = filter_cost_data_by_range(data, app.range);

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
    let item_colors = shared::create_color_mapping(&chart_data.items, palette);

    render_cost_chart(f, &filtered_data, area, &title, provider, &item_colors);
}
