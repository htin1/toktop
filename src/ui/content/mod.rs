mod cost;
mod shared;
mod usage;

use crate::app::{App, View};
use crate::ui::colors::ColorPalette;
use ratatui::layout::Rect;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    match app.current_view {
        View::Cost => cost::render_cost_view(f, app, area, provider, &palette),
        View::Usage => usage::render_usage_view(f, app, area, provider, &palette),
    }
}
