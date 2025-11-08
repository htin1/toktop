mod api;
mod app;
mod events;
mod fetch;
mod models;
mod provider;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(about = "A terminal-based LLM cost and usage monitor")]
struct Args {
    #[arg(short, long)]
    env_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    if let Some(env_file) = args.env_file {
        if let Err(e) = dotenvy::from_path(&env_file) {
            eprintln!("Warning: Failed to load env file '{}': {}", env_file.display(), e);
        }
    }

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

    spawn_fetch_task(app.clone());

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
            app_lock.update_animation_frame();
            terminal.draw(|f| ui::render(f, &app_lock))?;
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut app_lock = app.lock().await;
                    let action = events::handle_key_event(&mut app_lock, key.code);
                    drop(app_lock);

                    match action {
                        events::EventAction::Refresh => {
                            spawn_fetch_task(app.clone());
                        }
                        events::EventAction::Quit => break,
                        events::EventAction::None => {}
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

fn spawn_fetch_task(app: Arc<Mutex<App>>) {
    tokio::spawn(async move {
        let (provider, openai_client, anthropic_client) = {
            let mut app_lock = app.lock().await;
            let provider = app_lock.current_provider();

            if app_lock.loading {
                return;
            }

            app_lock.start_fetch();
            let (openai_client, anthropic_client) = app_lock.get_clients();
            (provider, openai_client, anthropic_client)
        };

        let outcome = fetch::fetch_data(provider, openai_client, anthropic_client).await;

        let mut app_lock = app.lock().await;
        app_lock.finish_fetch(outcome);
    });
}
