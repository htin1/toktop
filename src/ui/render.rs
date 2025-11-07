use crate::app::App;
use crate::ui::{content, footer, header, menu, popup, view};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    // Top % for header panel (menu + view + summary), remaining space minus footer for chart, footer at bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Top panel: split horizontally - left side (menu + view) and right side (summary)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(vertical_chunks[0]);

    // Top Left side: split vertically - menu on top, view on bottom
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(top_chunks[0]);

    menu::render(f, app, left_chunks[0]);
    view::render(f, app, left_chunks[1]);
    header::render(f, app, top_chunks[1]);

    // Middle section: full width chart
    content::render(f, app, vertical_chunks[1]);

    // Bottom: footer
    footer::render(f, app, vertical_chunks[2]);

    // Show popup overlay if loading or API key popup is active
    if app.loading || app.api_key_popup_active.is_some() {
        popup::render(f, app);
    }
}
