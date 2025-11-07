use crate::app::{App, Provider, View};
use crate::models::{DailyData, DailyUsageData};
use crate::ui::colors::ColorPalette;
use crate::ui::utils::format_tokens;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

// Constants
const DATE_LABEL_WIDTH: u16 = 8;
const DATE_LABEL_OFFSET: u16 = 9;
const BAR_PADDING: u16 = 20;
const LEGEND_WIDTH: u16 = 50;
const BAR_HEIGHT: u16 = 2;
const BAR_SPACING: u16 = 1;
const MIN_SEGMENT_WIDTH_FOR_TEXT: u16 = 6;
const MIN_SEGMENT_WIDTH_FOR_TOKENS: u16 = 10;
const COST_THRESHOLD_FOR_LEGEND: f64 = 1.0;

fn render_error_message(f: &mut Frame, area: Rect, title: &str, message: &str, color: Color) {
    f.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center)
            .style(Style::default().fg(color)),
        area,
    );
}

fn render_empty_state(f: &mut Frame, area: Rect, title: &str, message: &str) {
    f.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center),
        area,
    );
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    // Render the active view
    match app.current_view {
        View::Cost => render_cost_view(f, app, area, provider, &palette),
        View::Usage => render_usage_view(f, app, area, provider, &palette),
    }
}

fn render_cost_view(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider);
    let title = format!("{} - Daily Cost by Model", provider.label());

    if let Some(err) = error {
        render_error_message(
            f,
            area,
            &title,
            &format!("Error loading {} Cost data: {}", provider.label(), err),
            palette.error,
        );
        return;
    }

    if !has_client {
        render_empty_state(
            f,
            area,
            &title,
            &format!(
                "Connect an {} Admin API key to view this dashboard.",
                provider.label()
            ),
        );
        return;
    }

    let data = match app.data_for_provider(provider) {
        Some(values) => values,
        None => {
            render_empty_state(
                f,
                area,
                &title,
                &format!("{} Cost data is not wired up yet.", provider.label()),
            );
            return;
        }
    };

    if data.is_empty() && app.loading {
        render_empty_state(
            f,
            area,
            &title,
            &format!("Loading {} Cost data...", provider.label()),
        );
        return;
    }

    if data.is_empty() {
        render_empty_state(
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

    // Get usage data to create unified color mapping
    let usage_data = app.usage_data_for_provider(provider);

    // Create unified color mapping for models across both charts
    let unified_colors = create_unified_color_mapping(data, usage_data, palette);

    render_cost_chart(f, data, area, &title, provider, &unified_colors);
}

fn render_usage_view(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    palette: &ColorPalette,
) {
    let has_client = app.has_client(provider);
    let error = app.error_for_provider(provider);
    let title = format!("{} - Daily Token Usage by Model", provider.label());

    if let Some(err) = error {
        render_error_message(
            f,
            area,
            &title,
            &format!("Error loading {} Usage data: {}", provider.label(), err),
            palette.error,
        );
        return;
    }

    if !has_client {
        render_empty_state(
            f,
            area,
            &title,
            &format!(
                "Connect an {} Admin API key to view this dashboard.",
                provider.label()
            ),
        );
        return;
    }

    let usage_data = match app.usage_data_for_provider(provider) {
        Some(values) => values,
        None => {
            render_empty_state(
                f,
                area,
                &title,
                &format!("{} Usage data is not wired up yet.", provider.label()),
            );
            return;
        }
    };

    if usage_data.is_empty() && app.loading {
        render_empty_state(
            f,
            area,
            &title,
            &format!("Loading {} Usage data...", provider.label()),
        );
        return;
    }

    if usage_data.is_empty() {
        render_empty_state(
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

    // Get cost data to create unified color mapping
    let cost_data = app.data_for_provider(provider);

    // Create unified color mapping for models across both charts
    let unified_colors =
        create_unified_color_mapping(cost_data.unwrap_or(&[]), Some(usage_data), palette);

    render_usage_chart(f, app, area, provider, &unified_colors);
}

fn create_unified_color_mapping(
    cost_data: &[DailyData],
    usage_data: Option<&[DailyUsageData]>,
    palette: &ColorPalette,
) -> HashMap<String, Color> {
    let mut all_items = std::collections::HashSet::new();

    // Collect items from cost data
    for d in cost_data {
        if let Some(ref line_item) = d.line_item {
            let trimmed = line_item.trim();
            if !trimmed.is_empty() {
                all_items.insert(trimmed.to_string());
            }
        }
    }

    // Collect models from usage data
    if let Some(usage) = usage_data {
        for d in usage {
            if let Some(ref model) = d.model {
                let trimmed = model.trim();
                if !trimmed.is_empty() {
                    all_items.insert(trimmed.to_string());
                }
            }
        }
    }

    // Sort items for consistent color assignment
    let mut sorted_items: Vec<String> = all_items.into_iter().collect();
    sorted_items.sort();

    // Assign colors based on sorted order
    sorted_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            (
                item.clone(),
                palette.chart_colors[i % palette.chart_colors.len()],
            )
        })
        .collect()
}

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
        .filter(|item| item_totals.get(*item).copied().unwrap_or(0.0) > COST_THRESHOLD_FOR_LEGEND)
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

fn render_stacked_bar_segment(
    f: &mut Frame,
    area: Rect,
    text: &str,
    color: Color,
    text_color: Color,
) {
    f.render_widget(
        Paragraph::new(text).alignment(Alignment::Center).style(
            Style::default()
                .fg(text_color)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        area,
    );
}

fn render_cost_chart(
    f: &mut Frame,
    data: &[DailyData],
    area: Rect,
    title: &str,
    provider: Provider,
    unified_colors: &HashMap<String, Color>,
) {
    let palette = ColorPalette::for_provider(provider);
    let chart_data = process_cost_data(data);

    if chart_data.dates.is_empty() {
        render_empty_state(f, area, title, "No data available");
        return;
    }

    // Use unified colors, falling back to assigned colors for items not in unified mapping
    let mut item_colors = unified_colors.clone();
    for item in &chart_data.items {
        if !item_colors.contains_key(item) {
            // Fallback: assign color based on position in chart_data.items
            let index = chart_data.items.iter().position(|i| i == item).unwrap_or(0);
            item_colors.insert(
                item.clone(),
                palette.chart_colors[index % palette.chart_colors.len()],
            );
        }
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
        .constraints([Constraint::Min(0), Constraint::Length(LEGEND_WIDTH)])
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
    let total_height_per_bar = BAR_HEIGHT + BAR_SPACING;

    for (i, date) in chart_data.dates.iter().enumerate() {
        let y_pos = chart_area.y + (i as u16 * total_height_per_bar);

        if y_pos + BAR_HEIGHT > chart_area.y + chart_area.height {
            break;
        }

        let model_costs = &chart_data.daily_costs[date];
        let total_cost: f64 = model_costs.values().sum();

        // Date label
        let date_label_area = Rect::new(chart_area.x, y_pos, DATE_LABEL_WIDTH, BAR_HEIGHT);
        f.render_widget(
            Paragraph::new(date.clone()).style(Style::default().fg(Color::White)),
            date_label_area,
        );

        // Bar area
        let bar_x = chart_area.x + DATE_LABEL_OFFSET;
        let bar_width = chart_area.width.saturating_sub(BAR_PADDING);
        let mut current_x = bar_x;

        // Render stacked segments
        for item in &chart_data.items {
            if let Some(&cost) = model_costs.get(item) {
                if cost > 0.0 {
                    let segment_width = ((cost / max_total) * bar_width as f64) as u16;

                    if segment_width > 0 {
                        let color = item_colors.get(item).copied().unwrap_or(Color::White);
                        let segment_area = Rect::new(current_x, y_pos, segment_width, BAR_HEIGHT);

                        let text = if segment_width > MIN_SEGMENT_WIDTH_FOR_TEXT {
                            format!("${:.0}", cost)
                        } else {
                            "".to_string()
                        };

                        render_stacked_bar_segment(f, segment_area, &text, color, Color::Black);
                        current_x += segment_width;
                    }
                }
            }
        }

        // Total label
        let total_x = bar_x + bar_width + 2;
        let total_area = Rect::new(total_x, y_pos, 14, BAR_HEIGHT);
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

struct UsageChartData {
    daily_tokens: HashMap<String, HashMap<String, (u64, u64)>>,
    model_totals: HashMap<String, (u64, u64)>,
    dates: Vec<String>,
    models: Vec<String>,
}

fn process_usage_data(data: &[DailyUsageData]) -> UsageChartData {
    let mut daily_tokens: HashMap<String, HashMap<String, (u64, u64)>> = HashMap::new();
    let mut model_totals: HashMap<String, (u64, u64)> = HashMap::new();

    for d in data {
        let date_str = d.date.format("%m/%d").to_string();
        let model = d
            .model
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_string();

        let entry = daily_tokens
            .entry(date_str)
            .or_default()
            .entry(model.clone())
            .or_insert((0, 0));
        entry.0 += d.input_tokens;
        entry.1 += d.output_tokens;

        let total = model_totals.entry(model).or_insert((0, 0));
        total.0 += d.input_tokens;
        total.1 += d.output_tokens;
    }

    let mut dates: Vec<String> = daily_tokens.keys().cloned().collect();
    dates.sort();

    let mut models: Vec<String> = model_totals.keys().cloned().collect();
    models.sort();

    UsageChartData {
        daily_tokens,
        model_totals,
        dates,
        models,
    }
}

fn render_usage_legend(
    f: &mut Frame,
    area: Rect,
    models: &[String],
    model_totals: &HashMap<String, (u64, u64)>,
    model_colors: &HashMap<String, Color>,
    palette: &ColorPalette,
) {
    let mut legend_lines = vec![
        Line::from(Span::styled(
            "Models",
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for model in models {
        let color = model_colors.get(model).copied().unwrap_or(Color::White);
        let (input_total, output_total) = model_totals.get(model).copied().unwrap_or((0, 0));
        legend_lines.push(Line::from(vec![
            Span::styled(
                "   ",
                Style::default()
                    .bg(color)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(model.clone()),
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

fn render_usage_chart(
    f: &mut Frame,
    app: &App,
    area: Rect,
    provider: Provider,
    unified_colors: &HashMap<String, Color>,
) {
    let palette = ColorPalette::for_provider(provider);
    let title = format!("{} - Daily Token Usage by Model", provider.label());

    let usage_data = match app.usage_data_for_provider(provider) {
        Some(data) => data,
        None => {
            render_empty_state(f, area, &title, "Usage data not available");
            return;
        }
    };

    if usage_data.is_empty() {
        render_empty_state(f, area, &title, "No usage data available");
        return;
    }

    let chart_data = process_usage_data(usage_data);

    if chart_data.dates.is_empty() {
        render_empty_state(f, area, &title, "No data available");
        return;
    }

    // Use unified colors, falling back to assigned colors for models not in unified mapping
    let mut model_colors = unified_colors.clone();
    for model in &chart_data.models {
        if !model_colors.contains_key(model) {
            // Fallback: assign color based on position in chart_data.models
            let index = chart_data
                .models
                .iter()
                .position(|m| m == model)
                .unwrap_or(0);
            model_colors.insert(
                model.clone(),
                palette.chart_colors[index % palette.chart_colors.len()],
            );
        }
    }
    let max_total = chart_data
        .daily_tokens
        .values()
        .map(|models| {
            models
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
        .constraints([Constraint::Min(0), Constraint::Length(LEGEND_WIDTH)])
        .split(inner);

    render_usage_legend(
        f,
        chunks[1],
        &chart_data.models,
        &chart_data.model_totals,
        &model_colors,
        &palette,
    );

    let chart_area = chunks[0];
    let total_height_per_bar = BAR_HEIGHT + BAR_SPACING;

    for (i, date) in chart_data.dates.iter().enumerate() {
        let y_pos = chart_area.y + (i as u16 * total_height_per_bar);

        if y_pos + BAR_HEIGHT > chart_area.y + chart_area.height {
            break;
        }

        let model_tokens = &chart_data.daily_tokens[date];
        let total_tokens: u64 = model_tokens
            .values()
            .map(|(input, output)| input + output)
            .sum();

        // Date label
        let date_label_area = Rect::new(chart_area.x, y_pos, DATE_LABEL_WIDTH, BAR_HEIGHT);
        f.render_widget(
            Paragraph::new(date.clone()).style(Style::default().fg(Color::White)),
            date_label_area,
        );

        // Bar area
        let bar_x = chart_area.x + DATE_LABEL_OFFSET;
        let bar_width = chart_area.width.saturating_sub(BAR_PADDING);
        let mut current_x = bar_x;

        // Render stacked segments - one per model
        for model in &chart_data.models {
            if let Some(&(input_tokens, output_tokens)) = model_tokens.get(model) {
                let total_model_tokens = input_tokens + output_tokens;

                if total_model_tokens > 0 {
                    let segment_width =
                        ((total_model_tokens as f64 / max_total as f64) * bar_width as f64) as u16;

                    if segment_width > 0 {
                        let color = model_colors.get(model).copied().unwrap_or(Color::White);
                        let segment_area = Rect::new(current_x, y_pos, segment_width, BAR_HEIGHT);

                        let text = if segment_width > MIN_SEGMENT_WIDTH_FOR_TOKENS {
                            format_tokens(total_model_tokens)
                        } else {
                            "".to_string()
                        };

                        render_stacked_bar_segment(f, segment_area, &text, color, Color::Black);
                        current_x += segment_width;
                    }
                }
            }
        }

        // Total label
        let total_x = bar_x + bar_width + 2;
        let total_area = Rect::new(total_x, y_pos, 14, BAR_HEIGHT);
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
