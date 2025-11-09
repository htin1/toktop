use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::collections::HashMap;

pub const LEGEND_WIDTH: u16 = 50;
pub const COST_THRESHOLD_FOR_LEGEND: f64 = 1.0;
pub const VERTICAL_BAR_SPACING: u16 = 1;
pub const MAX_BAR_WIDTH: u16 = 20;
pub const HORIZONTAL_SCROLLBAR_HEIGHT: u16 = 1;

#[derive(Clone, Copy)]
pub struct VerticalBarLayout {
    pub start_index: usize,
    pub visible_bars: usize,
    pub bar_width: u16,
    pub spacing: u16,
    pub offset: u16,
}

pub fn vertical_bar_layout(
    total_bars: usize,
    area_width: u16,
    scroll_offset: usize,
) -> Option<VerticalBarLayout> {
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
        bar_width = bar_width.min(area_width).min(MAX_BAR_WIDTH);

        let required = visible * bar_width as usize + total_spacing;
        if required <= area_width as usize {
            let offset = ((area_width as usize - required) / 2) as u16;
            let max_scroll = total_bars.saturating_sub(visible);
            let start_index = scroll_offset.min(max_scroll);
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

pub fn render_horizontal_scrollbar(
    f: &mut Frame,
    area: Rect,
    total_items: usize,
    visible_items: usize,
    start_index: usize,
    accent_color: Color,
) {
    if total_items == 0
        || visible_items == 0
        || area.width == 0
        || area.height == 0
        || total_items <= visible_items
    {
        return;
    }

    let viewport = visible_items.max(1);
    let mut scrollbar_state = ScrollbarState::new(total_items)
        .content_length(total_items - visible_items)
        .viewport_content_length(viewport)
        .position(start_index);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some("─"))
        .track_style(Style::default().fg(Color::DarkGray))
        .thumb_symbol("━")
        .thumb_style(Style::default().fg(accent_color));

    f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
