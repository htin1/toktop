use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub const LEGEND_WIDTH: u16 = 50;
pub const COST_THRESHOLD_FOR_LEGEND: f64 = 1.0;
pub const VERTICAL_BAR_SPACING: u16 = 1;

#[derive(Clone, Copy)]
pub struct VerticalBarLayout {
    pub start_index: usize,
    pub visible_bars: usize,
    pub bar_width: u16,
    pub spacing: u16,
    pub offset: u16,
}

pub fn vertical_bar_layout(total_bars: usize, area_width: u16) -> Option<VerticalBarLayout> {
    if total_bars == 0 || area_width == 0 {
        return None;
    }

    let spacing = VERTICAL_BAR_SPACING;
    let min_bar_width: u16 = 5;
    let mut visible = total_bars.min(area_width as usize);

    while visible > 0 {
        let total_spacing = if visible > 1 {
            (visible - 1) * spacing as usize
        } else {
            0
        };

        if area_width as usize <= total_spacing {
            visible -= 1;
            continue;
        }

        let available_width = (area_width as usize).saturating_sub(total_spacing);
        let mut bar_width = (available_width / visible).max(min_bar_width as usize) as u16;
        bar_width = bar_width.min(area_width);

        let required = visible * bar_width as usize + total_spacing;
        if required <= area_width as usize {
            let offset = ((area_width as usize - required) / 2) as u16;
            let start_index = total_bars - visible;
            return Some(VerticalBarLayout {
                start_index,
                visible_bars: visible,
                bar_width,
                spacing,
                offset,
            });
        }

        visible -= 1;
    }

    None
}

pub fn compact_date_label(date: &str, width: u16) -> String {
    if width >= date.len() as u16 {
        return date.to_string();
    }

    let day_part = date.split('/').nth(1).unwrap_or(date);
    if width >= day_part.len() as u16 {
        return day_part.to_string();
    }

    day_part.chars().take(width as usize).collect::<String>()
}

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
    let mut sorted_items: Vec<String> = items.to_vec();
    sorted_items.sort();

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
