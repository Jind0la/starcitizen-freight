//! Freight — Star Citizen Cargo Profit Calculator
//!
//! Dead-simple: enter cargo SCU, get top 3 profit routes with fuel cost subtracted.

mod api;
mod calculation;
mod cli;
mod error;
mod models;

use anyhow::Result;
use chrono::Utc;
use cli::LOADING;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::AppState;
use std::io;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::api::UexClient;
use crate::calculation::rank_routes;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Initialize state
    let client = UexClient::new(std::env::var("UEX_API_TOKEN").ok());
    let mut app_state = AppState {
        cargo_scu: 0,
        routes: vec![],
        fuel_estimate: 0.0,
        last_updated: Utc::now(),
    };
    let mut input_value = String::new();
    let mut error_msg: Option<String> = None;
    let mut expanded_route: Option<u8> = None;
    let mut needs_redraw = true;

    // Main event loop
    loop {
        if needs_redraw {
            terminal.draw(|f| {
                cli::render(f, &app_state, &input_value, error_msg.as_deref(), expanded_route);
            })?;
            needs_redraw = false;
        }

        // Poll for events with timeout
        if crossterm::event::poll(Duration::from_millis(100))? {
            let evt = event::read()?;

            if let Event::Key(key) = evt {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Enter => {
                            if !input_value.is_empty() {
                                needs_redraw = calculate_and_render(
                                    &client,
                                    &input_value,
                                    &mut app_state,
                                    &mut error_msg,
                                ).await;
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            input_value.push(c);
                            error_msg = None;
                            needs_redraw = true;
                        }
                        KeyCode::Backspace => {
                            input_value.pop();
                            needs_redraw = true;
                        }
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Auto-refresh routes if we have them and cache expired
        if !app_state.routes.is_empty() {
            let age = Utc::now()
                .signed_duration_since(app_state.last_updated)
                .num_minutes();
            if age >= 30 {
                let cargo = app_state.cargo_scu;
                if cargo > 0 {
                    let _ = refresh_routes(&client, cargo, &mut app_state, &mut error_msg).await;
                    needs_redraw = true;
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Returns true if a redraw is needed.
async fn calculate_and_render(
    client: &UexClient,
    input_value: &str,
    app_state: &mut AppState,
    error_msg: &mut Option<String>,
) -> bool {
    let scu: u32 = match input_value.trim().parse() {
        Ok(n) if n > 0 && n <= 16000 => n,
        _ => {
            *error_msg = Some(format!("Invalid cargo size: '{}'. Enter 1-16000 SCU.", input_value));
            return true;
        }
    };

    LOADING.store(true, Ordering::SeqCst);
    let result = refresh_routes(client, scu, app_state, error_msg).await;
    LOADING.store(false, Ordering::SeqCst);

    result.is_ok()
}

async fn refresh_routes(
    client: &UexClient,
    cargo_scu: u32,
    app_state: &mut AppState,
    error_msg: &mut Option<String>,
) -> Result<(), error::AppError> {
    let (routes_result, fuel_result) = tokio::join!(client.get_routes(), client.get_fuel_prices());

    let routes = routes_result?;
    let fuel_prices = fuel_result.unwrap_or_default();

    let ranked = rank_routes(&routes, &fuel_prices, cargo_scu);
    let total_fuel: f64 = ranked.iter().map(|r| r.fuel_cost).sum();

    app_state.cargo_scu = cargo_scu;
    app_state.routes = ranked;
    app_state.fuel_estimate = total_fuel;
    app_state.last_updated = Utc::now();
    *error_msg = None;

    Ok(())
}
