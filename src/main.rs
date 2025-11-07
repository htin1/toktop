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

    // Spawn initial fetch (only if data doesn't exist)
    spawn_fetch_task(app.clone(), false);

    loop {
        // Check for missing API key and show popup if needed
        {
            let mut app_lock = app.lock().await;
            let current_provider = app_lock.current_provider();
            if !app_lock.has_client(current_provider) && app_lock.api_key_popup_active.is_none() {
                app_lock.show_api_key_popup(current_provider);
            }
        }

        // Render UI
        {
            let app_lock = app.lock().await;
            terminal.draw(|f| ui::render(f, &app_lock))?;
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut app_lock = app.lock().await;
                    let popup_active = app_lock.api_key_popup_active.is_some();

                    match key.code {
                        KeyCode::Up | KeyCode::Down => {
                            let delta = if key.code == KeyCode::Up { -1 } else { 1 };
                            app_lock.move_menu_cursor(delta);
                            if app_lock.select_menu_cursor() {
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
                        KeyCode::Tab => {
                            app_lock.toggle_view();
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
                    app::Provider::OpenAI => app_lock.openai_error = None,
                    app::Provider::Anthropic => app_lock.anthropic_error = None,
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

        let result = app::fetch_usage_data(provider, openai_client, anthropic_client).await;

        let mut app_lock = app.lock().await;
        match provider {
            app::Provider::OpenAI => {
                app_lock.data.openai = result.data.openai;
                app_lock.data.openai_usage = result.data.openai_usage;
                app_lock.openai_error = result.openai_error;
            }
            app::Provider::Anthropic => {
                app_lock.data.anthropic = result.data.anthropic;
                app_lock.data.anthropic_usage = result.data.anthropic_usage;
                app_lock.anthropic_error = result.anthropic_error;
            }
        }
        app_lock.loading = false;
    });
}
