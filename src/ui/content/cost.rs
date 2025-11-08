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
        .fold(0.0f64, f64::max)
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
    let total_height_per_bar = shared::BAR_HEIGHT + shared::BAR_SPACING;

    for (i, date) in chart_data.dates.iter().enumerate() {
        let y_pos = chart_area.y + (i as u16 * total_height_per_bar);

        if y_pos + shared::BAR_HEIGHT > chart_area.y + chart_area.height {
            break;
        }

        let model_costs = &chart_data.daily_costs[date];
        let total_cost: f64 = model_costs.values().sum();

        // Date label
        let date_label_area = Rect::new(
            chart_area.x,
            y_pos,
            shared::DATE_LABEL_WIDTH,
            shared::BAR_HEIGHT,
        );
        f.render_widget(
            Paragraph::new(date.clone()).style(Style::default().fg(Color::White)),
            date_label_area,
        );

        // Bar area
        let bar_x = chart_area.x + shared::DATE_LABEL_OFFSET;
        let bar_width = chart_area.width.saturating_sub(shared::BAR_PADDING);
        let mut current_x = bar_x;

        // Render stacked segments
        for item in &chart_data.items {
            if let Some(&cost) = model_costs.get(item) {
                if cost > 0.0 {
                    let segment_width = ((cost / max_total) * bar_width as f64) as u16;

                    if segment_width > 0 {
                        let color = item_colors.get(item).copied().unwrap_or(Color::White);
                        let segment_area =
                            Rect::new(current_x, y_pos, segment_width, shared::BAR_HEIGHT);

                        let text = if segment_width > shared::MIN_SEGMENT_WIDTH_FOR_TEXT {
                            format!("${:.0}", cost)
                        } else {
                            "".to_string()
                        };

                        shared::render_stacked_bar_segment(
                            f,
                            segment_area,
                            &text,
                            color,
                            Color::Black,
                        );
                        current_x += segment_width;
                    }
                }
            }
        }

        // Total label
        let total_x = bar_x + bar_width + 2;
        let total_area = Rect::new(total_x, y_pos, 14, shared::BAR_HEIGHT);
        f.render_widget(
            Paragraph::new(format!("${:.0}", total_cost)).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            total_area,
        );
    }
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

    // Create color mapping for cost chart items (always models)
    let chart_data = process_cost_data(data);
    let item_colors = shared::create_color_mapping(&chart_data.items, palette);

    render_cost_chart(f, data, area, &title, provider, &item_colors);
}
