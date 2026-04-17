//! TUI (Terminal User Interface) using ratatui.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::{RankedRoute, StockLevel};
use std::sync::atomic::{AtomicBool, Ordering};

pub static LOADING: AtomicBool = AtomicBool::new(false);

/// Render the main layout.
pub fn render(
    f: &mut Frame,
    state: &crate::models::AppState,
    input_value: &str,
    error_msg: Option<&str>,
    expanded_route: Option<u8>,
) {
    let chunks = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3),  // Header
            Constraint::Length(5),  // Input area
            Constraint::Min(10),  // Results
            Constraint::Length(3),  // Status bar
        ],
    )
    .split(f.area());

    render_header(f, chunks[0], &state.last_updated);
    render_input(f, chunks[1], input_value);
    render_results(f, chunks[2], &state.routes, expanded_route);
    render_statusbar(f, chunks[3], error_msg);
}

/// Header bar
fn render_header(f: &mut Frame, area: Rect, last_updated: &chrono::DateTime<chrono::Utc>) {
    let elapsed = chrono::Utc::now()
        .signed_duration_since(*last_updated);
    let time_str = if elapsed.num_minutes() < 1 {
        "just now".to_string()
    } else if elapsed.num_minutes() < 60 {
        format!("{}m ago", elapsed.num_minutes())
    } else {
        format!("{}h ago", elapsed.num_hours())
    };

    let text = Paragraph::new(Line::from(vec![
        Span::raw("FREIGHT "),
        Span::styled("v0.1.0", Color::DarkGray),
        Span::raw("  ·  Stanton System  ·  Updated "),
        Span::styled(time_str, Color::Blue),
    ]))
    .block(Block::new().borders(Borders::BOTTOM).border_style(Color::DarkGray))
    .style(Style::new().bg(Color::Black).fg(Color::White))
    .alignment(Alignment::Left);

    f.render_widget(text, area);
}

/// Input area: cargo SCU input + Calculate button
fn render_input(f: &mut Frame, area: Rect, input_value: &str) {
    let chunks = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ],
    )
    .split(area);

    // Left: label
    let label = Paragraph::new("CARGO (SCU)")
        .style(Style::new().fg(Color::DarkGray))
        .alignment(Alignment::Right);
    f.render_widget(label, chunks[0]);

    // Middle: text input display (styled box)
    let is_loading = LOADING.load(Ordering::SeqCst);

    let input_block = Block::new()
        .borders(Borders::ALL)
        .border_style(Color::Blue)
        .title(if is_loading { " Loading... " } else { " Enter SCU " });

    let input_style = if is_loading {
        Style::new().bg(Color::DarkGray).fg(Color::Black)
    } else {
        Style::new().bg(Color::Black).fg(Color::White)
    };

    let input_text = Paragraph::new(Line::from(vec![
        Span::raw(input_value),
        Span::raw("█"),
    ]))
    .block(input_block)
    .style(input_style)
    .alignment(Alignment::Left);

    f.render_widget(input_text, chunks[1]);

    // Right: Calculate button label
    let btn = Paragraph::new("[ CALCULATE ]")
        .style(Style::new().bg(Color::Blue).fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(btn, chunks[2]);
}

/// Results: ranked route cards
fn render_results(
    f: &mut Frame,
    area: Rect,
    routes: &[RankedRoute],
    expanded_route: Option<u8>,
) {
    if routes.is_empty() {
        let empty = Paragraph::new("Enter your cargo capacity above to find the best trade routes right now.")
            .style(Style::new().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::new().borders(Borders::NONE));
        f.render_widget(empty, area);
        return;
    }

    let constraints: Vec<Constraint> = routes
        .iter()
        .map(|_| Constraint::Min(8))
        .collect();

    let chunks = Layout::new(Direction::Vertical, constraints).split(area);

    for (i, route) in routes.iter().enumerate() {
        render_route_card(f, chunks[i], route, expanded_route == Some(route.rank));
    }
}

/// Single route card
fn render_route_card(f: &mut Frame, area: Rect, route: &RankedRoute, expanded: bool) {
    let stock_color = match route.stock_level {
        StockLevel::High => Color::Green,
        StockLevel::Medium => Color::Yellow,
        StockLevel::Low => Color::Red,
    };

    let stars = "★".repeat(route.stars as usize) + &"☆".repeat(3 - route.stars as usize);

    let route_line = Line::from(vec![
        Span::raw(format!("#{} ", route.rank)),
        Span::styled(stars, Color::Yellow),
        Span::raw(format!(
            "  {}  →  {}",
            truncate(&route.origin, 20),
            truncate(&route.destination, 20)
        )),
    ]);

    let commodity_line = Line::from(vec![
        Span::raw(route.commodity.clone()),
        Span::raw(format!(
            "    {} SCU @ {} → {}",
            route.scu_to_trade,
            fmt_num(route.buy_price),
            fmt_num(route.sell_price)
        )),
    ]);

    let profit_line = Line::from(vec![
        Span::styled(
            format!("+{} CR", fmt_num(route.total_profit)),
            Color::Green,
        ),
        Span::raw(format!(
            "  (+{}/SCU)   Margin: {}%   Stock: ",
            fmt_num(route.profit_per_scu),
            fmt_pct(route.margin_pct)
        )),
        Span::styled(route.stock_level.as_str(), stock_color),
    ]);

    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Color::DarkGray);

    let content = if expanded {
        vec![
            route_line,
            commodity_line,
            profit_line,
            Line::from(vec![Span::raw(format!(
                "  Fuel est: ~{} CR  ·  Distance: {} GM  ·  Containers: {}",
                fmt_num(route.fuel_cost),
                fmt_dist(route.distance_gm),
                route.container_sizes
            ))]),
            if let Some(days) = route.data_age_days {
                Line::from(vec![Span::raw(format!("  Data age: {} days", days))])
            } else {
                Line::from(vec![Span::raw("")])
            },
        ]
    } else {
        vec![route_line, commodity_line, profit_line]
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::new().bg(Color::Black).fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Status bar
fn render_statusbar(f: &mut Frame, area: Rect, error_msg: Option<&str>) {
    let status = if let Some(msg) = error_msg {
        Line::from(vec![
            Span::styled("● ", Color::Red),
            Span::raw(msg),
        ])
    } else {
        Line::from(vec![
            Span::styled("● ", Color::Green),
            Span::raw("UEX API connected  ·  Data: https://uexcorp.space"),
        ])
    };

    let bar = Paragraph::new(status)
        .block(Block::new().borders(Borders::TOP).border_style(Color::DarkGray))
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    f.render_widget(bar, area);
}

// ─── Formatting helpers ─────────────────────────────────────────────────────

/// Utility: truncate string to max width with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

/// Format large number with commas
fn fmt_num(n: f64) -> String {
    let abs_n = n.abs();
    let negative = n < 0.0;
    if abs_n >= 1000.0 {
        let s = abs_n as i64;
        let s = s.to_string();
        let formatted: String = s
            .chars()
            .rev()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 3 == 0 {
                    Some(',')
                } else {
                    None
                }
                .into_iter()
                .chain(std::iter::once(c))
            })
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if negative {
            format!("-{}", formatted)
        } else {
            formatted
        }
    } else {
        if negative {
            format!("-{:.0}", abs_n)
        } else {
            format!("{:.0}", abs_n)
        }
    }
}

/// Format as percentage integer
fn fmt_pct(n: f64) -> String {
    format!("{:.0}", n)
}

/// Format distance with 1 decimal
fn fmt_dist(n: f64) -> String {
    format!("{:.1}", n)
}
