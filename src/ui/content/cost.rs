use crate::app::{App, View};
use crate::models::DailyData;
use crate::provider::Provider;
use crate::ui::colors::ColorPalette;
use crate::ui::content::shared;
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

fn filter_items_by_cost_threshold(
    items: &[String],
    item_totals: &HashMap<String, f64>,
) -> Vec<String> {
    let mut filtered_items: Vec<String> = items
        .iter()
        .filter(|item| item_totals.get(*item).copied().unwrap_or(0.0) >= shared::COST_THRESHOLD)
        .cloned()
        .collect();

    if filtered_items.is_empty() {
        filtered_items = items.to_vec();
    }

    filtered_items
}

fn process_cost_data(data: &[DailyData]) -> CostChartData {
    let mut daily_costs: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut item_totals: HashMap<String, f64> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();
        let line_item = shared::extract_trimmed_string(&d.line_item)
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
    let legend_items = filter_items_by_cost_threshold(items, item_totals);

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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.primary).add_modifier(Modifier::DIM))
        .title(Span::styled(
            title,
            Style::default().fg(palette.primary).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

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

    let filtered_items = filter_items_by_cost_threshold(&chart_data.items, &chart_data.item_totals);
    let chart_items = &filtered_items;

    let chart_area = chunks[0];
    match shared::render_vertical_stacked_bars(
        f,
        chart_area,
        &chart_data.dates,
        chart_items,
        |date, item| {
            chart_data
                .daily_costs
                .get(date)
                .and_then(|items| items.get(item).copied())
        },
        |date| {
            chart_data
                .daily_costs
                .get(date)
                .map(|items| items.values().sum())
                .unwrap_or(0.0)
        },
        |total| format!("${:.0}", total),
        |value| {
            if value >= 1.0 {
                format!("${:.0}", value)
            } else if value >= 0.1 {
                format!("${:.1}", value)
            } else {
                format!("${:.2}", value)
            }
        },
        item_colors,
        max_total,
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
                "Not enough space to render cost chart",
            );
            None
        }
    }
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
        app.filter_cost_data_by_range(data)
    };

    if range_filtered_data.is_empty() {
        let msg = if app.loading {
            format!("Loading {} Cost data...", provider.label())
        } else {
            format!(
                "No {} Cost data available for the selected window.",
                provider.label()
            )
        };
        shared::render_empty_state(f, area, &title, &msg);
        return;
    }

    let all_items_chart_data = process_cost_data(&range_filtered_data);
    let all_item_colors = shared::create_color_mapping(&all_items_chart_data.items, palette);

    let filtered_data =
        shared::apply_filter(&range_filtered_data, app.selected_filter.as_ref(), |d| {
            shared::extract_trimmed_string(&d.line_item)
        });

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
    let item_colors = shared::filter_item_colors(&all_item_colors, &chart_data.items);

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
