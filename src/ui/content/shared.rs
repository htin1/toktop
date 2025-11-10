use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::collections::HashMap;

pub const LEGEND_WIDTH: u16 = 50;
pub const COST_THRESHOLD: f64 = 1.0;
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

pub fn extract_trimmed_string(opt: &Option<String>) -> Option<&str> {
    opt.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty())
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

pub fn filter_item_colors(
    all_colors: &HashMap<String, Color>,
    filtered_items: &[String],
) -> HashMap<String, Color> {
    filtered_items
        .iter()
        .filter_map(|item| all_colors.get(item).map(|color| (item.clone(), *color)))
        .collect()
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

pub fn render_stacked_bar_segment_with_value(
    f: &mut Frame,
    area: Rect,
    value_text: &str,
    color: Color,
) {
    f.render_widget(
        Paragraph::new(value_text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Gray).bg(color)),
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

pub fn render_vertical_stacked_bars<F, G>(
    f: &mut Frame,
    chart_area: Rect,
    dates: &[String],
    items: &[String],
    get_value: F,
    get_total: G,
    format_total: impl Fn(f64) -> String,
    format_segment_value: impl Fn(f64) -> String,
    item_colors: &HashMap<String, Color>,
    max_total: f64,
    scroll_offset: usize,
    show_segment_values: bool,
) -> Option<VerticalBarLayout>
where
    F: Fn(&str, &str) -> Option<f64>,
    G: Fn(&str) -> f64,
{
    if chart_area.width == 0 || chart_area.height <= 1 || max_total <= 0.0 {
        return None;
    }

    let label_height: u16 = 1;
    let value_label_height: u16 = 1;
    let scrollbar_height = HORIZONTAL_SCROLLBAR_HEIGHT;
    let bar_area_height = chart_area
        .height
        .saturating_sub(label_height)
        .saturating_sub(value_label_height)
        .saturating_sub(scrollbar_height);
    if bar_area_height == 0 {
        return None;
    }
    let bars_y = chart_area.y + value_label_height;

    let layout = match vertical_bar_layout(dates.len(), chart_area.width, scroll_offset) {
        Some(layout) => layout,
        None => return None,
    };

    let end_index = layout.start_index + layout.visible_bars;

    for (visible_idx, date_idx) in (layout.start_index..end_index).enumerate() {
        let date = &dates[date_idx];
        let total = get_total(date);
        let bar_x = chart_area.x
            + layout.offset
            + (visible_idx as u16) * (layout.bar_width + layout.spacing);

        let mut used_height = 0;
        let mut top_segment_area: Option<Rect> = None;
        for item in items {
            if let Some(value) = get_value(date, item) {
                if value <= 0.0 {
                    continue;
                }

                let mut segment_height =
                    ((value / max_total) * bar_area_height as f64).round() as u16;
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
                if show_segment_values {
                    let value_text = format_segment_value(value);
                    render_stacked_bar_segment_with_value(f, segment_area, &value_text, color);
                } else {
                    render_stacked_bar_segment(f, segment_area, "", color, Color::Black);
                }
                top_segment_area = Some(segment_area);
                used_height += segment_height;
            }
        }

        if used_height == 0 && bar_area_height > 0 {
            let marker_y = bars_y + bar_area_height - 1;
            render_stacked_bar_segment(
                f,
                Rect::new(bar_x, marker_y, layout.bar_width, 1),
                "",
                Color::DarkGray,
                Color::Black,
            );
        }

        if total > 0.0 {
            if let Some(segment_area) = top_segment_area {
                let label_y = segment_area.y - 1;
                f.render_widget(
                    Paragraph::new(format_total(total))
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
        let label_text = compact_date_label(date, layout.bar_width);
        f.render_widget(
            Paragraph::new(label_text).alignment(Alignment::Center),
            label_area,
        );
    }

    Some(layout)
}

pub fn handle_chart_scrollbar(
    f: &mut Frame,
    app: &mut crate::app::App,
    chart_area: Rect,
    total_dates: usize,
    layout: VerticalBarLayout,
    accent_color: Color,
) {
    let scrollbar_visible =
        total_dates > layout.visible_bars && chart_area.height >= HORIZONTAL_SCROLLBAR_HEIGHT;
    app.chart_scrollbar_visible = scrollbar_visible;

    if scrollbar_visible {
        let scrollbar_height = HORIZONTAL_SCROLLBAR_HEIGHT.min(chart_area.height);
        let scrollbar_area = Rect::new(
            chart_area.x,
            chart_area.y + chart_area.height.saturating_sub(scrollbar_height),
            chart_area.width,
            scrollbar_height,
        );
        render_horizontal_scrollbar(
            f,
            scrollbar_area,
            total_dates,
            layout.visible_bars,
            layout.start_index,
            accent_color,
        );
    }
}

pub fn apply_filter<T: Clone>(
    data: &[T],
    selected_filter: Option<&String>,
    extract_field: impl Fn(&T) -> Option<&str>,
) -> Vec<T> {
    if let Some(filter) = selected_filter {
        data.iter()
            .filter(|d| {
                extract_field(d)
                    .map(|s| s == filter.as_str())
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    } else {
        data.to_vec()
    }
}
