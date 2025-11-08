mod api;
mod app;
mod models;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> io::Result<()> {
    let openai_key = std::env::var("OPENAI_ADMIN_KEY").ok();
    let anthropic_key = std::env::var("ANTHROPIC_ADMIN_KEY").ok();

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();
    if let Some(key) = openai_key {
        app.set_openai_client(key);
    }
    if let Some(key) = anthropic_key {
        app.set_anthropic_client(key);
    }

    let app = Arc::new(Mutex::new(app));

    spawn_fetch_task(app.clone(), false);

    loop {
        {
            let mut app_lock = app.lock().await;
            let current_provider = app_lock.current_provider();
            if !app_lock.has_client(current_provider) && app_lock.api_key_popup_active.is_none() {
                app_lock.show_api_key_popup(current_provider);
            }
        }

        // Render UI
        {
            let mut app_lock = app.lock().await;
            let provider = app_lock.current_provider();
            let has_data = match provider {
                app::Provider::OpenAI => !app_lock.data.openai.is_empty(),
                app::Provider::Anthropic => !app_lock.data.anthropic.is_empty(),
            };

            if app_lock.loading || !has_data {
                app_lock.animation_frame = app_lock.animation_frame.wrapping_add(1);
            } else {
                app_lock.animation_frame = 0;
            }
            terminal.draw(|f| ui::render(f, &app_lock))?;
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut app_lock = app.lock().await;
                    let popup_active = app_lock.api_key_popup_active.is_some();

                    match key.code {
                        KeyCode::Left | KeyCode::Right => {
                            let delta = if key.code == KeyCode::Left { -1 } else { 1 };
                            app_lock.move_options_column(delta);
                        }
                        KeyCode::Up | KeyCode::Down => {
                            let delta = if key.code == KeyCode::Up { -1 } else { 1 };
                            let provider_before = app_lock.current_provider();
                            app_lock.move_column_cursor(delta);
                            let provider_changed = provider_before != app_lock.current_provider();
                            if provider_changed {
                                let new_provider = app_lock.current_provider();
                                if !app_lock.has_client(new_provider) {
                                    app_lock.show_api_key_popup(new_provider);
                                } else {
                                    app_lock.cancel_api_key_popup();
                                }
                                drop(app_lock);
                                spawn_fetch_task(app.clone(), false);
                            }
                        }
                        KeyCode::Enter if popup_active => {
                            if app_lock.submit_api_key() {
                                drop(app_lock);
                                spawn_fetch_task(app.clone(), false);
                            }
                        }
                        KeyCode::Esc if popup_active => {
                            app_lock.cancel_api_key_popup();
                        }
                        _ if popup_active => {
                            app_lock.handle_api_key_input(key.code);
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            drop(app_lock);
                            spawn_fetch_task(app.clone(), true);
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    }
                }
            }
        } else {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn spawn_fetch_task(app: Arc<Mutex<App>>, force_refresh: bool) {
    tokio::spawn(async move {
        let (provider, openai_client, anthropic_client, should_fetch) = {
            let mut app_lock = app.lock().await;
            let provider = app_lock.current_provider();

            if !app_lock.has_client(provider) {
                return;
            }

            let data_exists = match provider {
                app::Provider::OpenAI => !app_lock.data.openai.is_empty(),
                app::Provider::Anthropic => !app_lock.data.anthropic.is_empty(),
            };

            let should_fetch = !data_exists || force_refresh;
            if should_fetch {
                app_lock.loading = true;
                match provider {
                    app::Provider::OpenAI => {
                        app_lock.openai_errors = app::ProviderErrors::default();
                    }
                    app::Provider::Anthropic => {
                        app_lock.anthropic_errors = app::ProviderErrors::default();
                    }
                }
            }

            (
                provider,
                app_lock.openai_client.clone(),
                app_lock.anthropic_client.clone(),
                should_fetch,
            )
        };

        if !should_fetch {
            return;
        }

        let result = app::fetch_data(provider, openai_client, anthropic_client).await;

        let mut app_lock = app.lock().await;
        let app::FetchOutcome {
            data,
            openai_errors,
            anthropic_errors,
        } = result;
        let crate::models::UsageData {
            openai,
            anthropic,
            anthropic_usage,
            openai_usage,
            anthropic_api_key_names,
            openai_api_key_names,
        } = data;
        match provider {
            app::Provider::OpenAI => {
                app_lock.data.openai = openai;
                app_lock.data.openai_usage = openai_usage;
                app_lock.data.openai_api_key_names = openai_api_key_names;
                app_lock.openai_errors = openai_errors;
            }
            app::Provider::Anthropic => {
                app_lock.data.anthropic = anthropic;
                app_lock.data.anthropic_usage = anthropic_usage;
                app_lock.data.anthropic_api_key_names = anthropic_api_key_names;
                app_lock.anthropic_errors = anthropic_errors;
            }
        }
        app_lock.loading = false;
    });
}
