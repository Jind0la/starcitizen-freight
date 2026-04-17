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
// Commodities
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct Commodity {
    pub id: u32,
    #[serde(rename = "id_parent")]
    pub parent_id: Option<u32>,
    pub name: String,
    pub code: Option<String>,
    pub kind: Option<String>,
    #[serde(rename = "weight_scu")]
    pub weight_scu: Option<f64>,
    #[serde(rename = "price_buy")]
    pub price_buy: Option<f64>,
    #[serde(rename = "price_sell")]
    pub price_sell: Option<f64>,
    #[serde(rename = "is_available")]
    pub is_available: Option<i32>,
    #[serde(rename = "is_available_live")]
    pub is_available_live: Option<i32>,
    #[serde(rename = "is_visible")]
    pub is_visible: Option<i32>,
    #[serde(rename = "is_buyable")]
    pub is_buyable: Option<i32>,
    #[serde(rename = "is_sellable")]
    pub is_sellable: Option<i32>,
    #[serde(rename = "is_illegal")]
    pub is_illegal: Option<i32>,
    #[serde(rename = "is_fuel")]
    pub is_fuel: Option<i32>,
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
    #[serde(rename = "id_orbit_origin")]
    pub orbit_origin_id: Option<u32>,
    #[serde(rename = "id_orbit_destination")]
    pub orbit_destination_id: Option<u32>,
    #[serde(rename = "id_terminal_origin")]
    pub terminal_origin_id: u32,
    #[serde(rename = "id_terminal_destination")]
    pub terminal_destination_id: u32,
    #[serde(rename = "id_faction_origin")]
    pub faction_origin_id: Option<u32>,
    #[serde(rename = "id_faction_destination")]
    pub faction_destination_id: Option<u32>,
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
    pub scu_origin_users: Option<f64>,
    pub scu_origin_users_rows: Option<f64>,
    pub scu_destination: Option<f64>,
    pub scu_destination_users: Option<f64>,
    pub scu_destination_users_rows: Option<f64>,
    pub scu_margin: Option<f64>,
    pub scu_reachable: Option<f64>,
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
    #[serde(rename = "origin_terminal_is_player_owned")]
    pub origin_terminal_is_player_owned: Option<i32>,
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
    pub destination_terminal_slug: Option<String>,
    #[serde(rename = "destination_terminal_is_player_owned")]
    pub destination_terminal_is_player_owned: Option<i32>,
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
pub struct FuelEntry {
    pub id: u32,
    #[serde(rename = "id_commodity")]
    pub commodity_id: u32,
    #[serde(rename = "id_star_system")]
    pub star_system_id: u32,
    #[serde(rename = "id_orbit")]
    pub orbit_id: Option<u32>,
    #[serde(rename = "id_terminal")]
    pub terminal_id: u32,
    pub price_buy: f64,
    pub price_buy_avg: Option<f64>,
    #[serde(rename = "commodity_name")]
    pub commodity_name: String,
    #[serde(rename = "star_system_name")]
    pub star_system_name: Option<String>,
    #[serde(rename = "orbit_name")]
    pub orbit_name: Option<String>,
    #[serde(rename = "terminal_name")]
    pub terminal_name: String,
}

impl FuelEntry {
    /// Returns the effective hydrogen fuel price (avg if available, else spot).
    pub fn effective_price(&self) -> f64 {
        self.price_buy_avg.unwrap_or(self.price_buy)
    }
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
// Star Systems
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct StarSystem {
    pub id: u32,
    pub name: String,
    pub code: Option<String>,
    #[serde(rename = "is_available")]
    pub is_available: Option<i32>,
    #[serde(rename = "is_available_live")]
    pub is_available_live: Option<i32>,
    #[serde(rename = "is_visible")]
    pub is_visible: Option<i32>,
    #[serde(rename = "is_default")]
    pub is_default: Option<i32>,
    #[serde(rename = "faction_name")]
    pub faction_name: Option<String>,
}

/// Known system IDs in UEX API
pub const SYSTEM_ID_STANTON: u32 = 68;
pub const SYSTEM_ID_PYRO: u32 = 64;
pub const SYSTEM_ID_NYX: u32 = 55;

/// System name constants used by the UEX API.
pub const SYSTEM_NAME_STANTON: &str = "Stanton";
pub const SYSTEM_NAME_PYRO: &str = "Pyro";
pub const SYSTEM_NAME_NYX: &str = "Nyx";

/// All star systems available in Freight (is_visible = 1 AND is_available_live = 1).
pub const AVAILABLE_SYSTEMS: &[(u32, &str)] = &[
    (SYSTEM_ID_STANTON, SYSTEM_NAME_STANTON),
    (SYSTEM_ID_PYRO, SYSTEM_NAME_PYRO),
    (SYSTEM_ID_NYX, SYSTEM_NAME_NYX),
];

// Lagrange orbit IDs that act as jump gates between Stanton ↔ Pyro
// These appear in the orbits_distances data
pub const LAGRANGE_STANTON_TERRA_GATEWAY: u32 = 361; // Terra Gateway in Stanton
pub const LAGRANGE_STANTON_MICROTECH_L1: u32 = 339; // microTech Lagrange Point 1
pub const LAGRANGE_STANTON_ARCCORP_L1: u32 = 326;   // ArcCorp Lagrange Point 1
pub const LAGRANGE_STANTON_CRUSAHER_L5: u32 = 333;  // Crusader Lagrange Point 5
pub const STANTON_GATEWAY_TO_PYRO: u32 = 398;       // Stanton Gateway (Pyro system) → in Stanton view
pub const PYRO_GATEWAY_STANTON: u32 = 398;          // Same orbit, different system perspective

// ============================================================================
// Orbit Distances (for cross-system route calculation)
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct OrbitDistance {
    #[serde(rename = "id_star_system_origin")]
    pub star_system_origin_id: u32,
    #[serde(rename = "id_star_system_destination")]
    pub star_system_destination_id: u32,
    #[serde(rename = "id_orbit_origin")]
    pub orbit_origin_id: u32,
    #[serde(rename = "id_orbit_destination")]
    pub orbit_destination_id: u32,
    pub distance: f64,
    #[serde(rename = "orbit_origin_name")]
    pub orbit_origin_name: Option<String>,
    #[serde(rename = "orbit_destination_name")]
    pub orbit_destination_name: Option<String>,
    #[serde(rename = "star_system_name")]
    pub star_system_name: Option<String>,
}

// ============================================================================
// Application domain models (enriched/processed)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Number of quantum jumps between systems for a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum JumpCount {
    Zero,  // intra-system
    One,   // jump to Lagrange/gateway
    Two,   // jump through Lagrange
}

impl JumpCount {
    pub fn as_u8(&self) -> u8 {
        match self {
            JumpCount::Zero => 0,
            JumpCount::One => 1,
            JumpCount::Two => 2,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            JumpCount::Zero => "",
            JumpCount::One => "1 jump",
            JumpCount::Two => "2 jumps",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedRoute {
    pub rank: u8,
    pub stars: u8,
    pub commodity: String,
    pub commodity_slug: Option<String>,
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
    /// Whether the origin terminal is player-owned
    pub is_player_owned: bool,
    /// The terminal slug for the destination (for Q-Link navigation)
    pub destination_slug: Option<String>,
    /// Whether this route crosses star systems (Stanton ↔ Pyro, etc.)
    pub is_interstellar: bool,
    /// Number of quantum jumps required (0=intra-system, 1-2=interstellar)
    pub jump_count: u8,
    /// The destination star system name (for interstellar routes)
    pub destination_system: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppState {
    pub cargo_scu: u32,
    pub routes: Vec<RankedRoute>,
    pub fuel_estimate: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Ships — built-in cargo capacity reference
// ============================================================================

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Ship {
    pub name: &'static str,
    pub scu: u32,
    pub max_container_size: u32, // 1=SCU, 2=ME, 4=SE, 8=TE, 16=BE, 32=AE, 0=no cargo
}

pub const SHIPS: &[Ship] = &[
    // Small (1-32 SCU)
    Ship { name: "Aurora MR",          scu: 0,   max_container_size: 0 },
    Ship { name: "Aurora LX",          scu: 0,   max_container_size: 0 },
    Ship { name: "Mustang Alpha",       scu: 0,   max_container_size: 0 },
    Ship { name: "Cutter",             scu: 0,   max_container_size: 0 },
    Ship { name: "Nomad",              scu: 6,   max_container_size: 1 },
    Ship { name: "Nostromo",           scu: 8,   max_container_size: 1 },
    Ship { name: "300i",              scu: 0,   max_container_size: 0 },
    Ship { name: "315p",              scu: 0,   max_container_size: 0 },

    // Medium (33-400 SCU)
    Ship { name: "Cutlass Black",      scu: 66,  max_container_size: 4 },
    Ship { name: "Cutlass Red",        scu: 66,  max_container_size: 4 },
    Ship { name: "Cutlass Blue",       scu: 66,  max_container_size: 4 },
    Ship { name: "Freelancer",         scu: 120, max_container_size: 4 },
    Ship { name: "Freelancer MAX",     scu: 120, max_container_size: 4 },
    Ship { name: "Freelancer DUR",     scu: 120, max_container_size: 4 },
    Ship { name: "Constellation Taurus", scu: 256, max_container_size: 8 },
    Ship { name: "Constellation Andromeda", scu: 256, max_container_size: 8 },
    Ship { name: "Constellation Aquila",  scu: 256, max_container_size: 8 },
    Ship { name: "Constellation Phoenix",  scu: 256, max_container_size: 8 },
    Ship { name: "Corsair",            scu: 180, max_container_size: 8 },
    Ship { name: "Spirit C1",          scu: 128, max_container_size: 4 },
    Ship { name: "Valkyrie",          scu: 160, max_container_size: 4 },

    // Large (401-2000 SCU)
    Ship { name: "Caterpillar",        scu: 576, max_container_size: 16 },
    Ship { name: "Caterpillar Solo",  scu: 288, max_container_size: 16 },
    Ship { name: "Mole",               scu: 96,  max_container_size: 2 },
    Ship { name: "Mole Carbon",        scu: 96,  max_container_size: 2 },
    Ship { name: "Mole Talonite",      scu: 96,  max_container_size: 2 },
    Ship { name: "Mole Avalon",        scu: 96,  max_container_size: 2 },
    Ship { name: "Hull A",            scu: 200, max_container_size: 4 },
    Ship { name: "Hull B",            scu: 500, max_container_size: 8 },
    Ship { name: "Hull C",            scu: 1500, max_container_size: 16 },
    Ship { name: "Hull D",            scu: 2000, max_container_size: 32 },
    Ship { name: "Starfarer",          scu: 400, max_container_size: 16 },
    Ship { name: "Starfarer Gemini",   scu: 400, max_container_size: 16 },
    Ship { name: "Reclaimer",          scu: 180, max_container_size: 4 },
    Ship { name: "Rescue",             scu: 0,   max_container_size: 0 },

    // Extra Large (2001+ SCU)
    Ship { name: "500 Tons",          scu: 5000, max_container_size: 32 },
    Ship { name: "800 Tons",          scu: 8000, max_container_size: 32 },
    Ship { name: "Carrack",           scu: 240, max_container_size: 4 },
    Ship { name: "Expanse",           scu: 3200, max_container_size: 32 },
    Ship { name: "MPUV Cargo",        scu: 1,   max_container_size: 1 },
    Ship { name: "X1 Velocity",       scu: 0,   max_container_size: 0 },
];
