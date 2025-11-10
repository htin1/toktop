use crate::app::App;
use crate::ui::{content, footer, summary, options, popup};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    // Top panel (options + summary)
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Top panel: split horizontally - left side (options) and right side (summary)
    {
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(vertical_chunks[0]);
        let app_ref = &*app;
        options::render(f, app_ref, top_chunks[0]);
        summary::render(f, app_ref, top_chunks[1]);
    }

    // Middle section: full width chart
    content::render(f, app, vertical_chunks[1]);

    // Bottom: footer
    {
        let app_ref = &*app;
        footer::render(f, app_ref, vertical_chunks[2]);
        // Show popup overlay if loading or API key popup is active
        if app_ref.loading || app_ref.api_key_popup_active.is_some() {
            popup::render(f, app_ref);
        }
    }
}
