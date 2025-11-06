use crate::app::App;
use crate::ui::{content, footer, header, menu, popup};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    // Top 20% for header panel (menu + summary), remaining space minus footer for chart, footer at bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());
    
    // Top panel: split horizontally - menu on left, summary on right
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(vertical_chunks[0]);
    
    menu::render(f, app, top_chunks[0]);
    header::render(f, app, top_chunks[1]);
    
    // Middle section: full width chart
    content::render(f, app, vertical_chunks[1]);
    footer::render(f, app, vertical_chunks[2]);

    // Show loading popup overlay if loading
    if app.loading {
        popup::render(f, app);
    }
}

