use crate::app::{App, Provider};
use crate::models::DailyData;
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider);
    let title = format!("{} - Daily Cost by Line Item", provider.label());

    if let Some(err) = error {
        f.render_widget(
            Paragraph::new(format!(
                "Error loading {} Cost data: {}",
                provider.label(),
                err
            ))
            .block(Block::default().borders(Borders::ALL).title(title.clone()))
            .alignment(Alignment::Center)
            .style(Style::default().fg(palette.error)),
            area,
        );
        return;
    }

    if !has_client {
        f.render_widget(
            Paragraph::new(format!(
                "Connect an {} Admin API key to view this dashboard.",
                provider.label()
            ))
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let data = match app.data_for_provider(provider) {
        Some(values) => values,
        None => {
            f.render_widget(
                Paragraph::new(format!(
                    "{} Cost data is not wired up yet.",
                    provider.label()
                ))
                .block(Block::default().borders(Borders::ALL).title(title))
                .alignment(Alignment::Center),
                area,
            );
            return;
        }
    };

    if data.is_empty() && app.loading {
        f.render_widget(
            Paragraph::new(format!(
                "Loading {} Cost data...",
                provider.label()
            ))
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }

    if data.is_empty() {
        f.render_widget(
            Paragraph::new(format!(
                "No {} Cost data available for the selected window.",
                provider.label()
            ))
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }

    render_stacked_chart(f, data, area, &title, provider);
}

fn render_stacked_chart(f: &mut Frame, data: &[DailyData], area: Rect, title: &str, provider: Provider) {
    let palette = ColorPalette::for_provider(provider);
    // Group by date and collect totals per model so colors/legend stay in sync
    let mut daily_model_costs: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut line_item_totals: HashMap<String, f64> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();
        let raw_name = d
            .line_item
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown");
        let line_item = raw_name.to_string();

        *daily_model_costs
            .entry(date_str)
            .or_insert_with(HashMap::new)
            .entry(line_item.clone())
            .or_insert(0.0) += d.cost;

        *line_item_totals.entry(line_item).or_insert(0.0) += d.cost;
    }

    // Sort dates
    let mut dates: Vec<_> = daily_model_costs.keys().cloned().collect();
    dates.sort();

    if dates.is_empty() {
        f.render_widget(
            Paragraph::new("No data available")
                .block(Block::default().borders(Borders::ALL).title(title))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    // Get all line items sorted
    let mut all_line_items: Vec<String> = line_item_totals.keys().cloned().collect();
    all_line_items.sort();

    // Assign colors to ALL line items using provider-specific palette
    let line_item_colors: HashMap<String, Color> = all_line_items
        .iter()
        .enumerate()
        .map(|(i, item)| (item.clone(), palette.chart_colors[i % palette.chart_colors.len()]))
        .collect();

    // Filter legend items to only show models that cost > $1
    let threshold = 1.0;
    let mut legend_items: Vec<String> = all_line_items
        .iter()
        .filter(|item| line_item_totals.get(*item).copied().unwrap_or(0.0) > threshold)
        .cloned()
        .collect();
    if legend_items.is_empty() {
        legend_items = all_line_items.clone();
    }

    // Calculate max total for scaling
    let max_total = daily_model_costs
        .values()
        .map(|models| models.values().sum::<f64>())
        .fold(0.0f64, f64::max)
        .max(1.0);

    let block = Block::default().borders(Borders::ALL).title(title);

    f.render_widget(block, area);
    let inner = Block::default().borders(Borders::ALL).inner(area);

    // Reserve space for legend on the right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(50)])
        .split(inner);

    let chart_area = chunks[0];
    let legend_area = chunks[1];

    // Render legend (only significant models > $1)
    let mut legend_lines = vec![
        Line::from(Span::styled(
            "Models (>$1)",
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for line_item in &legend_items {
        let color = line_item_colors
            .get(line_item)
            .copied()
            .unwrap_or(Color::White);
        legend_lines.push(Line::from(vec![
            Span::styled(
                "   ",
                Style::default()
                    .bg(color)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(line_item.clone()),
        ]));
    }

    f.render_widget(
        Paragraph::new(legend_lines).alignment(Alignment::Left),
        legend_area,
    );

    // Calculate bar height
    let bar_height = 2;
    let bar_spacing = 1;
    let total_height_per_bar = bar_height + bar_spacing;

    // Render each date as a horizontal stacked bar
    for (i, date) in dates.iter().enumerate() {
        let y_pos = chart_area.y + (i as u16 * total_height_per_bar);

        if y_pos + bar_height > chart_area.y + chart_area.height {
            break;
        }

        let model_costs = &daily_model_costs[date];
        let total_cost: f64 = model_costs.values().sum();

        // Date label
        let date_label_area = Rect::new(chart_area.x, y_pos, 8, bar_height);
        f.render_widget(
            Paragraph::new(date.clone()).style(Style::default().fg(Color::White)),
            date_label_area,
        );

        // Bar area
        let bar_x = chart_area.x + 9;
        let bar_width = chart_area.width.saturating_sub(20);

        // Render stacked segments
        let mut current_x = bar_x;

        for line_item in &all_line_items {
            if let Some(&cost) = model_costs.get(line_item) {
                if cost > 0.0 {
                    let segment_width = ((cost / max_total) * bar_width as f64) as u16;

                    if segment_width > 0 {
                        let color = line_item_colors
                            .get(line_item)
                            .copied()
                            .unwrap_or(Color::White);
                        let segment_area = Rect::new(current_x, y_pos, segment_width, bar_height);

                        let text = if segment_width > 6 {
                            format!("${:.0}", cost)
                        } else {
                            "".to_string()
                        };

                        f.render_widget(
                            Paragraph::new(text).alignment(Alignment::Center).style(
                                Style::default()
                                    .fg(Color::Black)
                                    .bg(color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            segment_area,
                        );

                        current_x += segment_width;
                    }
                }
            }
        }

        // Total label on the right
        let total_x = bar_x + bar_width + 2;
        let total_area = Rect::new(total_x, y_pos, 14, bar_height);
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

