use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

// Constants
pub const DATE_LABEL_WIDTH: u16 = 8;
pub const DATE_LABEL_OFFSET: u16 = 9;
pub const BAR_PADDING: u16 = 20;
pub const LEGEND_WIDTH: u16 = 50;
pub const BAR_HEIGHT: u16 = 2;
pub const BAR_SPACING: u16 = 1;
pub const MIN_SEGMENT_WIDTH_FOR_TEXT: u16 = 6;
pub const MIN_SEGMENT_WIDTH_FOR_TOKENS: u16 = 10;
pub const COST_THRESHOLD_FOR_LEGEND: f64 = 1.0;

pub fn render_error_message(f: &mut Frame, area: Rect, title: &str, message: &str, color: Color) {
    f.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(color)),
        area,
    );
}

pub fn render_empty_state(f: &mut Frame, area: Rect, title: &str, message: &str) {
    f.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

pub fn create_color_mapping(items: &[String], palette: &ColorPalette) -> HashMap<String, Color> {
    // Sort items for consistent color assignment
    let mut sorted_items: Vec<String> = items.to_vec();
    sorted_items.sort();

    // Assign colors by rotating through palette
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

pub fn abbreviate_api_key(id: &str) -> String {
    if id.chars().count() <= 16 {
        return id.to_string();
    }

    let prefix: String = id.chars().take(8).collect();
    let suffix: String = id
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();

    format!("{}...{}", prefix, suffix)
}

pub fn render_stacked_bar_segment(
    f: &mut Frame,
    area: Rect,
    text: &str,
    color: Color,
    text_color: Color,
) {
    f.render_widget(
        Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(
                Style::default()
                    .fg(text_color)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
    );
}
