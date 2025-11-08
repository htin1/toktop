use crate::app::App;
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    f.render_widget(
        Paragraph::new(vec![Line::from(vec![
            Span::raw("Commands: "),
            Span::styled("←/→/↑/↓", Style::default().fg(palette.accent)),
            Span::raw("=switch option "),
            Span::raw("| "),
            Span::styled("r", Style::default().fg(palette.primary)),
            Span::raw("=refresh "),
            Span::raw("| "),
            Span::styled("q", Style::default().fg(palette.error)),
            Span::raw("=quit"),
        ])])
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}
