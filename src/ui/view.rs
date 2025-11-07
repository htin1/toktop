use crate::app::{App, View};
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);
    let cost_active = app.current_view == View::Cost;
    let usage_active = app.current_view == View::Usage;

    let cost_style = if cost_active {
        Style::default()
            .fg(palette.primary)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let usage_style = if usage_active {
        Style::default()
            .fg(palette.primary)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let tabs = vec![Line::from(vec![
        Span::styled(" Cost ", cost_style),
        Span::raw(" "),
        Span::styled(" Usage ", usage_style),
    ])];

    f.render_widget(
        Paragraph::new(tabs)
            .block(Block::default().borders(Borders::ALL).title("View"))
            .alignment(ratatui::layout::Alignment::Left),
        area,
    );
}
