use crate::app::App;
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    
    if let Some(popup_provider) = app.api_key_popup_active {
        render_api_key_popup(f, area, popup_provider, &app.api_key_input);
    } else if app.loading {
        let provider = app.current_provider();
        let palette = ColorPalette::for_provider(provider);
        render_loading_popup(f, area, palette);
    }
}

fn create_centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

fn create_popup_block(title: &str, palette: ColorPalette) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.primary))
        .title(title)
        .title_style(Style::default().fg(palette.primary).add_modifier(Modifier::BOLD))
}

fn render_loading_popup(f: &mut Frame, area: Rect, palette: ColorPalette) {
    let popup_area = create_centered_popup(area, 40, 5);
    let block = create_popup_block(" Loading ", palette);
    let inner = block.inner(popup_area);

    f.render_widget(block, popup_area);
    f.render_widget(
        Paragraph::new("Fetching usage data...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner,
    );
}

fn render_api_key_popup(f: &mut Frame, area: Rect, provider: crate::app::Provider, input_text: &str) {
    let palette = ColorPalette::for_provider(provider);
    let popup_area = create_centered_popup(area, 60, 6);
    let title = format!(" Enter {} API Key ", provider.label());
    let block = create_popup_block(&title, palette);
    let inner = block.inner(popup_area);

    f.render_widget(block, popup_area);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("{}_", input_text),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to submit, Esc to cancel",
                Style::default().fg(Color::White),
            )),
        ])
        .alignment(Alignment::Left),
        inner,
    );
}

