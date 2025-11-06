mod api;
mod app;
mod models;
mod ui;

use app::{prompt_for_key, App};
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
    let openai_key = std::env::var("OPENAI_ADMIN_KEY").ok().or_else(|| {
        if std::env::var("ANTHROPIC_ADMIN_KEY").is_ok() {
            None
        } else {
            Some(prompt_for_key("OpenAI"))
        }
    });
    let anthropic_key = std::env::var("ANTHROPIC_ADMIN_KEY").ok().or_else(|| {
        if openai_key.is_some() {
            None
        } else {
            Some(prompt_for_key("Anthropic"))
        }
    });
    if openai_key.is_none() && anthropic_key.is_none() {
        eprintln!("Error: At least one API key required");
        return Ok(());
    }

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
                    match key.code {
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            drop(app_lock);
                            spawn_fetch_task(app.clone(), true);
                        }
                        KeyCode::Up => {
                            app_lock.move_menu_cursor(-1);
                            let provider_changed = app_lock.select_menu_cursor();
                            drop(app_lock);
                            if provider_changed {
                                spawn_fetch_task(app.clone(), false);
                            }
                        }
                        KeyCode::Down => {
                            app_lock.move_menu_cursor(1);
                            let provider_changed = app_lock.select_menu_cursor();
                            drop(app_lock);
                            if provider_changed {
                                spawn_fetch_task(app.clone(), false);
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    }
                }
            }
        } else {
            // No event, just continue loop to re-render
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
            
            // Check if data already exists for this provider
            let data_exists = match provider {
                app::Provider::OpenAI => !app_lock.data.openai.is_empty(),
                app::Provider::Anthropic => !app_lock.data.anthropic.is_empty(),
            };
            
            // Only fetch if data doesn't exist or if forced refresh
            let should_fetch = !data_exists || force_refresh;
            
            if should_fetch {
                app_lock.loading = true;
                // Clear error for the current provider only
                match provider {
                    app::Provider::OpenAI => {
                        app_lock.openai_error = None;
                    }
                    app::Provider::Anthropic => {
                        app_lock.anthropic_error = None;
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

        let result = app::fetch_usage_data(provider, openai_client, anthropic_client).await;

        let mut app_lock = app.lock().await;
        // Only update data for the provider we fetched
        match provider {
            app::Provider::OpenAI => {
                app_lock.data.openai = result.data.openai;
                app_lock.openai_error = result.openai_error;
            }
            app::Provider::Anthropic => {
                app_lock.data.anthropic = result.data.anthropic;
                app_lock.anthropic_error = result.anthropic_error;
            }
        }
        app_lock.loading = false;
    });
}
