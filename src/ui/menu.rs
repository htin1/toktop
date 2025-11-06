use crate::app::{App, Provider};
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let providers = [Provider::OpenAI, Provider::Anthropic];
    let mut lines = Vec::new();
    
    for provider in providers.iter() {
        let has_client = app.has_client(*provider);
        let is_selected = app.current_provider() == *provider;
        let palette = ColorPalette::for_provider(*provider);
        
        let prefix = if is_selected { "> " } else { "  " };
        let mut label = provider.label().to_string();
        if !has_client {
            label.push_str(" (key needed)");
        }
        
        let style = if is_selected {
            Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else if !has_client {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let padded = format!("{prefix}{label}");
        lines.push(Line::from(Span::styled(padded, style)));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Providers")
                    .title_alignment(Alignment::Center),
            )
            .alignment(Alignment::Left),
        area,
    );
}

