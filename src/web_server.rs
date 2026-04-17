//! Axum web server for Freight

use crate::api::UexClient;
use crate::calculation::rank_routes;
use crate::error::AppError;
use crate::models::{RankedRoute, SYSTEM_ID_STANTON};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppServices {
    client: UexClient,
}

#[derive(Debug, Deserialize)]
struct RouteParams {
    scu: Option<u32>,
    system_id: Option<u32>,
    ship_max_container: Option<u32>,
}

#[derive(serde::Serialize)]
struct RoutesResponse {
    routes: Vec<RankedRoute>,
    total_fuel_estimate: f64,
    last_updated: String,
    cached: bool,
    cache_age_ms: Option<u64>,
}

/// GET /api/routes?scu=500&system_id=68&ship_max_container=8
async fn routes_handler(
    Query(params): Query<RouteParams>,
    State(services): State<Arc<AppServices>>,
) -> Result<Json<RoutesResponse>, AppError> {
    let scu = params.scu.unwrap_or(500);
    let system_id = params.system_id.unwrap_or(SYSTEM_ID_STANTON);
    let ship_max_container = params.ship_max_container;

    // Fetch all data
    let (all_routes, commodities) = tokio::join!(
        services.client.get_routes(),
        services.client.get_commodities()
    );

    let all_routes = all_routes?;
    let commodities = commodities?;

    // Rank and filter
    let ranked = rank_routes(&all_routes, &commodities, scu, ship_max_container, system_id);

    // Sum fuel costs for summary
    let total_fuel_estimate: f64 = ranked.iter().map(|r| r.fuel_cost).sum();

    let last_updated = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M UTC")
        .to_string();

    Ok(Json(RoutesResponse {
        routes: ranked,
        total_fuel_estimate,
        last_updated,
        cached: false,
        cache_age_ms: None,
    }))
}

/// Serve the web UI (index.html + static assets via rust-embed)
async fn serve_index() -> impl IntoResponse {
    serve_static("index.html", "text/html")
}

async fn serve_styles() -> impl IntoResponse {
    serve_static("styles.css", "text/css")
}

async fn serve_app_js() -> impl IntoResponse {
    serve_static("app.js", "application/javascript")
}

/// Serve a built-in static asset
fn serve_static(file: &str, mime: &str) -> Response {
    let embed = crate::web_ui::get(file);
    let body: Body = match embed {
        Some(data) => data.data.into_owned().into(),
        None => {
            return (StatusCode::NOT_FOUND, "Not found").into_response();
        }
    };
    let mut res = Response::new(body);
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        mime.parse().unwrap_or("application/octet-stream".parse().unwrap()),
    );
    res.headers_mut().insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    res
}

/// Serve favicon
async fn serve_favicon() -> impl IntoResponse {
    serve_static("favicon.svg", "image/svg+xml")
}

pub async fn start_web_server(client: UexClient, port: u16) {
    let services = Arc::new(AppServices { client });

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_styles))
        .route("/app.js", get(serve_app_js))
        .route("/favicon.svg", get(serve_favicon))
        .route("/api/routes", get(routes_handler))
        .layer(cors)
        .with_state(services);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🌐 Web UI listening on http://localhost:{port}");

    axum::serve(listener, app)
        .await
        .expect("web server error");
}
