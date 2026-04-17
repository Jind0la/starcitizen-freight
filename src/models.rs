//! API response models for UEX API 2.0

use serde::Deserialize;

// ============================================================================
// API Response Wrappers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: T,
}

// ============================================================================
// Commodity Routes
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub id: u32,
    #[serde(rename = "id_commodity")]
    pub commodity_id: u32,
    #[serde(rename = "id_star_system_origin")]
    pub star_system_origin_id: u32,
    #[serde(rename = "id_star_system_destination")]
    pub star_system_destination_id: u32,
    #[serde(rename = "id_planet_origin")]
    pub planet_origin_id: Option<u32>,
    #[serde(rename = "id_planet_destination")]
    pub planet_destination_id: Option<u32>,
    #[serde(rename = "id_terminal_origin")]
    pub terminal_origin_id: u32,
    #[serde(rename = "id_terminal_destination")]
    pub terminal_destination_id: u32,
    pub code: String,
    pub price_origin: f64,
    pub price_destination: f64,
    pub price_origin_users: Option<f64>,
    pub price_destination_users: Option<f64>,
    pub price_origin_users_rows: Option<f64>,
    pub price_destination_users_rows: Option<f64>,
    pub price_margin: f64,
    pub price_roi: f64,
    pub scu_origin: Option<f64>,
    pub scu_destination: Option<f64>,
    pub scu_margin: Option<f64>,
    pub scu_origin_users: Option<f64>,
    pub scu_destination_users: Option<f64>,
    pub volatility_origin: Option<f64>,
    pub volatility_destination: Option<f64>,
    pub status_origin: Option<i32>,
    pub status_destination: Option<i32>,
    pub investment: f64,
    pub profit: f64,
    pub distance: Option<f64>,
    pub score: Option<f64>,
    #[serde(rename = "container_sizes_origin")]
    pub container_sizes_origin: Option<String>,
    #[serde(rename = "container_sizes_destination")]
    pub container_sizes_destination: Option<String>,
    #[serde(rename = "has_docking_port_origin")]
    pub has_docking_port_origin: Option<i32>,
    #[serde(rename = "has_docking_port_destination")]
    pub has_docking_port_destination: Option<i32>,
    #[serde(rename = "has_freight_elevator_origin")]
    pub has_freight_elevator_origin: Option<i32>,
    #[serde(rename = "has_freight_elevator_destination")]
    pub has_freight_elevator_destination: Option<i32>,
    #[serde(rename = "has_loading_dock_origin")]
    pub has_loading_dock_origin: Option<i32>,
    #[serde(rename = "has_loading_dock_destination")]
    pub has_loading_dock_destination: Option<i32>,
    #[serde(rename = "has_refuel_origin")]
    pub has_refuel_origin: Option<i32>,
    #[serde(rename = "has_refuel_destination")]
    pub has_refuel_destination: Option<i32>,
    #[serde(rename = "has_cargo_center_origin")]
    pub has_cargo_center_origin: Option<i32>,
    #[serde(rename = "has_cargo_center_destination")]
    pub has_cargo_center_destination: Option<i32>,
    #[serde(rename = "has_quantum_marker_origin")]
    pub has_quantum_marker_origin: Option<i32>,
    #[serde(rename = "has_quantum_marker_destination")]
    pub has_quantum_marker_destination: Option<i32>,
    #[serde(rename = "is_monitored_origin")]
    pub is_monitored_origin: Option<i32>,
    #[serde(rename = "is_monitored_destination")]
    pub is_monitored_destination: Option<i32>,
    #[serde(rename = "is_space_station_origin")]
    pub is_space_station_origin: Option<i32>,
    #[serde(rename = "is_space_station_destination")]
    pub is_space_station_destination: Option<i32>,
    #[serde(rename = "is_on_ground_origin")]
    pub is_on_ground_origin: Option<i32>,
    #[serde(rename = "is_on_ground_destination")]
    pub is_on_ground_destination: Option<i32>,
    #[serde(rename = "commodity_name")]
    pub commodity_name: String,
    #[serde(rename = "commodity_code")]
    pub commodity_code: Option<String>,
    #[serde(rename = "commodity_slug")]
    pub commodity_slug: Option<String>,
    #[serde(rename = "origin_star_system_name")]
    pub origin_star_system_name: Option<String>,
    #[serde(rename = "origin_planet_name")]
    pub origin_planet_name: Option<String>,
    #[serde(rename = "origin_orbit_name")]
    pub origin_orbit_name: Option<String>,
    #[serde(rename = "origin_terminal_name")]
    pub terminal_origin_name: String,
    #[serde(rename = "origin_terminal_code")]
    pub terminal_origin_code: Option<String>,
    #[serde(rename = "origin_terminal_slug")]
    pub terminal_origin_slug: Option<String>,
    #[serde(rename = "origin_faction_name")]
    pub origin_faction_name: Option<String>,
    #[serde(rename = "destination_star_system_name")]
    pub destination_star_system_name: Option<String>,
    #[serde(rename = "destination_planet_name")]
    pub destination_planet_name: Option<String>,
    #[serde(rename = "destination_orbit_name")]
    pub destination_orbit_name: Option<String>,
    #[serde(rename = "destination_terminal_name")]
    pub terminal_destination_name: String,
    #[serde(rename = "destination_terminal_code")]
    pub terminal_destination_code: Option<String>,
    #[serde(rename = "destination_terminal_slug")]
    pub terminal_destination_slug: Option<String>,
    #[serde(rename = "destination_faction_name")]
    pub destination_faction_name: Option<String>,
    #[serde(rename = "game_version_origin")]
    pub game_version_origin: Option<String>,
    #[serde(rename = "game_version_destination")]
    pub game_version_destination: Option<String>,
    #[serde(rename = "date_added")]
    pub date_added: Option<u64>,
}

// ============================================================================
// Fuel Prices
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct FuelPrice {
    pub id: u32,
    #[serde(rename = "id_commodity")]
    pub commodity_id: u32,
    #[serde(rename = "id_star_system")]
    pub star_system_id: u32,
    #[serde(rename = "id_planet")]
    pub planet_id: Option<u32>,
    #[serde(rename = "id_orbit")]
    pub orbit_id: Option<u32>,
    #[serde(rename = "id_terminal")]
    pub terminal_id: u32,
    pub price_buy: f64,
    pub price_buy_min: Option<f64>,
    pub price_buy_min_week: Option<f64>,
    pub price_buy_min_month: Option<f64>,
    pub price_buy_max: Option<f64>,
    pub price_buy_max_week: Option<f64>,
    pub price_buy_max_month: Option<f64>,
    pub price_buy_avg: Option<f64>,
    pub price_buy_avg_week: Option<f64>,
    pub price_buy_avg_month: Option<f64>,
    #[serde(rename = "commodity_name")]
    pub commodity_name: String,
    #[serde(rename = "commodity_code")]
    pub commodity_code: Option<String>,
    #[serde(rename = "star_system_name")]
    pub star_system_name: Option<String>,
    #[serde(rename = "planet_name")]
    pub planet_name: Option<String>,
    #[serde(rename = "orbit_name")]
    pub orbit_name: Option<String>,
    #[serde(rename = "terminal_name")]
    pub terminal_name: String,
    #[serde(rename = "terminal_code")]
    pub terminal_code: Option<String>,
}

// ============================================================================
// Terminals
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct Terminal {
    pub id: u32,
    #[serde(rename = "id_star_system")]
    pub star_system_id: Option<u32>,
    #[serde(rename = "id_planet")]
    pub planet_id: Option<u32>,
    #[serde(rename = "id_space_station")]
    pub space_station_id: Option<u32>,
    pub name: String,
    pub fullname: Option<String>,
    #[serde(rename = "type")]
    pub terminal_type: Option<String>,
    pub code: Option<String>,
    pub screenshot: Option<String>,
    #[serde(rename = "is_available")]
    pub is_available: Option<i32>,
    #[serde(rename = "is_available_live")]
    pub is_available_live: Option<i32>,
    #[serde(rename = "is_visible")]
    pub is_visible: Option<i32>,
    #[serde(rename = "is_cargo_center")]
    pub is_cargo_center: Option<i32>,
    #[serde(rename = "has_loading_dock")]
    pub has_loading_dock: Option<i32>,
    #[serde(rename = "has_docking_port")]
    pub has_docking_port: Option<i32>,
    #[serde(rename = "has_freight_elevator")]
    pub has_freight_elevator: Option<i32>,
    #[serde(rename = "max_container_size")]
    pub max_container_size: Option<i32>,
    #[serde(rename = "star_system_name")]
    pub star_system_name: Option<String>,
    #[serde(rename = "planet_name")]
    pub planet_name: Option<String>,
    #[serde(rename = "space_station_name")]
    pub space_station_name: Option<String>,
    #[serde(rename = "faction_name")]
    pub faction_name: Option<String>,
}

// ============================================================================
// Application domain models (enriched/processed)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockLevel {
    High,
    Medium,
    Low,
}

impl StockLevel {
    pub fn from_status(status: Option<i32>) -> Self {
        match status {
            Some(s) if s >= 70 => StockLevel::High,
            Some(s) if s >= 30 => StockLevel::Medium,
            _ => StockLevel::Low,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StockLevel::High => "HIGH",
            StockLevel::Medium => "MED",
            StockLevel::Low => "LOW",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankedRoute {
    pub rank: u8,
    pub stars: u8,
    pub commodity: String,
    pub origin: String,
    pub destination: String,
    pub scu_to_trade: u32,
    pub buy_price: f64,
    pub sell_price: f64,
    pub total_profit: f64,
    pub profit_per_scu: f64,
    pub margin_pct: f64,
    pub stock_level: StockLevel,
    pub fuel_cost: f64,
    pub container_sizes: String,
    pub distance_gm: f64,
    pub data_age_days: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub cargo_scu: u32,
    pub routes: Vec<RankedRoute>,
    pub fuel_estimate: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
