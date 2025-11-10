use crate::app::{App, OptionsColumn};
use crossterm::event::KeyCode;

pub enum EventAction {
    Refresh,
    Quit,
    None,
}

pub fn handle_key_event(app: &mut App, key_code: KeyCode) -> EventAction {
    let popup_active = app.api_key_popup_active.is_some();

    match key_code {
        KeyCode::Left | KeyCode::Right => {
            let delta = if key_code == KeyCode::Left { -1 } else { 1 };
            app.move_options_column(delta);
            EventAction::None
        }
        KeyCode::Up | KeyCode::Down => {
            let delta = if key_code == KeyCode::Up { -1 } else { 1 };
            let provider_before = app.current_provider();
            app.move_column_cursor(delta);

            if provider_before != app.current_provider() {
                let new_provider = app.current_provider();
                if !app.has_client(new_provider) {
                    app.show_api_key_popup(new_provider);
                } else {
                    app.cancel_api_key_popup();
                    if !app.initial_fetch_done(new_provider) {
                        return EventAction::Refresh;
                    }
                }
            }
            EventAction::None
        }
        KeyCode::Enter if popup_active => {
            if app.submit_api_key() {
                EventAction::Refresh
            } else {
                EventAction::None
            }
        }
        KeyCode::Enter if !popup_active && app.options_column == OptionsColumn::GroupBy => {
            app.toggle_group_by_expansion();
            EventAction::None
        }
        KeyCode::Esc if popup_active => EventAction::Quit,
        _ if popup_active => {
            app.handle_api_key_input(key_code);
            EventAction::None
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.scroll_chart(-1);
            EventAction::None
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.scroll_chart(1);
            EventAction::None
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.toggle_segment_values();
            EventAction::None
        }
        KeyCode::Char('r') | KeyCode::Char('R') => EventAction::Refresh,
        KeyCode::Char('q') | KeyCode::Char('Q') => EventAction::Quit,
        _ => EventAction::None,
    }
}
