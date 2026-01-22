//! Flux TUI - A minimalist terminal interface for Flux CD
//!
//! This application provides a user-friendly TUI for managing Flux CD resources
//! including Kustomizations, HelmReleases, and HelmCharts.

mod app;
mod event;
mod kubernetes;
mod ui;

use std::io;
use std::time::Duration;

use app::{actions::Action, state::App};
use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

/// Tick rate for the event loop (controls refresh rate)
const TICK_RATE: Duration = Duration::from_millis(250);

/// Data refresh interval
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// A minimalist TUI for managing Flux CD resources
#[derive(Parser)]
#[command(name = "flux-tui")]
#[command(version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments (handles --version and --help automatically)
    Args::parse();

    // Initialize error handling
    color_eyre::install()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(false)
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle any errors from the app
    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

/// Main application loop implementing The Elm Architecture
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_refresh = std::time::Instant::now();

    loop {
        // Draw the UI
        terminal.draw(|frame| ui::draw::draw(frame, &app))?;

        // Auto-refresh data periodically
        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh_data().await?;
            last_refresh = std::time::Instant::now();
        }

        // Poll for events with timeout
        if poll(TICK_RATE)? {
            if let Event::Key(key) = read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    // Convert key event to action
                    let action = event::handler::handle_key_event(key, &app);

                    // Update state based on action
                    match action {
                        Action::Quit => return Ok(()),
                        action => app.update(action).await?,
                    }
                }
            }
        }
    }
}
