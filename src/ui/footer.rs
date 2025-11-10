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

    let mut spans = vec![
        Span::raw("Commands: "),
        Span::styled("←/→/↑/↓", Style::default().fg(palette.accent)),
        Span::raw("=switch option "),
    ];

    if app.chart_scrollbar_visible {
        spans.push(Span::raw("| "));
        spans.push(Span::styled("h/l", Style::default().fg(palette.accent)));
        spans.push(Span::raw("=scroll chart "));
    }

    spans.push(Span::raw("| "));
    spans.push(Span::styled("d", Style::default().fg(palette.accent)));
    spans.push(Span::raw("=toggle details "));

    spans.push(Span::raw("| "));
    spans.push(Span::styled("r", Style::default().fg(palette.primary)));
    spans.push(Span::raw("=refresh "));
    spans.push(Span::raw("| "));
    spans.push(Span::styled("q", Style::default().fg(palette.error)));
    spans.push(Span::raw("=quit"));

    f.render_widget(
        Paragraph::new(vec![Line::from(spans)])
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}
