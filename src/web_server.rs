//! Freight Web UI Server
//!
//! Serves the web UI and provides the /api/routes endpoint.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode, header},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use rust_embed::Embed;

use crate::api::UexClient;
use crate::calculation::rank_routes;
use crate::error::AppError;
use crate::models::RankedRoute;

// Embed static files from src/web/
#[derive(Embed)]
#[folder = "src/web"]
struct StaticAssets;

// ---------------------------------------------------------------------------
// API Types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct RoutesQuery {
    scu: u32,
}

#[derive(serde::Serialize)]
pub struct RoutesResponse {
    routes: Vec<RankedRoute>,
    total_fuel_estimate: f64,
    last_updated: String,
}

// ---------------------------------------------------------------------------
// Static file handling
// ---------------------------------------------------------------------------

fn serve_static(filename: &str) -> Response<Body> {
    let data = if filename == "index.html" || filename.is_empty() {
        StaticAssets::get("index.html").map(|f| f.data.into_owned())
    } else {
        StaticAssets::get(filename).map(|f| f.data.into_owned())
    };

    match data {
        Some(data) => {
            let mime = mime_guess::from_path(filename)
                .first_or_octet_stream();
            Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(data))
                .unwrap()
        }
        None => {
            if let Some(file) = StaticAssets::get("index.html") {
                Response::builder()
                    .status(200)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(file.data.into_owned()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

async fn routes_handler(
    State(client): State<Arc<UexClient>>,
    Query(query): Query<RoutesQuery>,
) -> Result<axum::Json<RoutesResponse>, AppError> {
    if query.scu == 0 || query.scu > 16000 {
        return Err(AppError::InvalidInput(
            "SCU must be between 1 and 16000".into(),
        ));
    }

    let (routes_result, fuel_result) = tokio::join!(
        client.get_routes(),
        client.get_fuel_prices()
    );

    let routes = routes_result?;
    let fuel_prices = fuel_result.unwrap_or_default();

    let ranked = rank_routes(&routes, &fuel_prices, query.scu);
    let total_fuel: f64 = ranked.iter().map(|r| r.fuel_cost).sum();

    Ok(axum::Json(RoutesResponse {
        routes: ranked,
        total_fuel_estimate: total_fuel,
        last_updated: chrono::Utc::now().format("%H:%M UTC").to_string(),
    }))
}

async fn static_handler(Path(filename): Path<String>) -> Response<Body> {
    serve_static(&filename)
}

async fn static_index() -> Response<Body> {
    serve_static("index.html")
}

async fn static_app_js() -> Response<Body> {
    serve_static("app.js")
}

async fn static_styles_css() -> Response<Body> {
    serve_static("styles.css")
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

pub async fn start_server(port: u16, token: Option<String>) -> anyhow::Result<()> {
    let client = Arc::new(UexClient::new(token));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/routes", get(routes_handler))
        .route("/app.js", get(static_app_js))
        .route("/styles.css", get(static_styles_css))
        .route("/static/{filename}", get(static_handler))
        .route("/", get(static_index))
        .layer(cors)
        .with_state(client);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!();
    println!("  ╔═══════════════════════════════════════════╗");
    println!("  ║       ◈ FREIGHT — Web UI Ready          ║");
    println!("  ╠═══════════════════════════════════════════╣");
    println!("  ║  🌐  http://localhost:{}", port);
    println!("  ║                                           ║");
    println!("  ║  Press Ctrl+C to stop                    ║");
    println!("  ╚═══════════════════════════════════════════╝");
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
