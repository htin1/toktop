use crate::app::App;
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    // Create centered popup area
    let popup_width = 40;
    let popup_height = 5;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.primary))
        .title(" Loading ")
        .title_style(
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(popup_area);

    f.render_widget(block, popup_area);

    f.render_widget(
        Paragraph::new("Fetching usage data...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner,
    );
}

