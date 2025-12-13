//! TUI front-end entry (Ratatui + Crossterm)
//! - Creates the DB client (SQLite direct)
//! - Sets up terminal

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::database::db::{migrate, queries};

pub mod api;
pub mod state;
pub mod input;
pub mod util;
pub mod ui;


pub async fn run() -> Result<()> {

    let mut app = init_app().await?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    app.refresh_accounts().await?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
                app.handle_key(key).await?;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn init_app() -> Result<state::App> {
    // Load database URL
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./finance_tracker.db".to_string());

    // Create DB client
    let client = api::Client::sqlite(&db_url).await?;
    
    let pool = client.pool();
    // Run migrations
    migrate::run_migrations(pool).await?;

    // Seed categories
    queries::seed_fixed_categories(pool).await?;

    // Create app state
    let app = state::App::new(client);

    Ok(app)
}