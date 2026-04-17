//! Route filtering, ranking, and profit calculation logic.

use crate::models::{
    Commodity, RankedRoute, Route, StockLevel, SYSTEM_NAME_NYX, SYSTEM_NAME_PYRO,
    SYSTEM_NAME_STANTON, SYSTEM_ID_STANTON,
};

/// Default hydrogen price (CR per SCU) when API data is unavailable.
/// Real in-game price is ~700-900 aUEC per SCU Hydrogen.
const DEFAULT_HYDROGEN_PRICE: f64 = 800.0;

/// Estimated hydrogen SCU consumed per 100 GM of quantum travel.
const FUEL_CONSUMPTION_PER_100GM: f64 = 10.0;

/// Round-trip multiplier for fuel calculation.
const ROUND_TRIP_MULTIPLIER: f64 = 2.0;

/// Minimum price margin (%) to consider a route.
const MIN_MARGIN_PCT: f64 = 5.0;

/// Maximum routes to return.
const MAX_RESULTS: usize = 3;

/// Extra fuel cost in SCU H2 per quantum jump (jump point traversal).
const QUANTUM_FUEL_PER_JUMP: f64 = 30.0;

/// Known Lagrange orbit IDs in Stanton that connect to the Pyro jump gate.
const STANTON_LAGRANGE_ORBITS: &[u32] = &[
    339, // microTech Lagrange Point 1
    326, // ArcCorp Lagrange Point 1
    333, // Crusader Lagrange Point 5
    361, // Terra Gateway
    116, // Hurston → Pyro VI Lagrange Point 5
    59,  // Crusader
];

/// Star rating thresholds.
fn calculate_stars(route: &Route, _data_age_days: Option<u32>) -> u8 {
    let mut stars = 1u8;
    if route.score.unwrap_or(0.0) >= 7.0 {
        stars += 1;
    }
    if let Some(rows) = route.price_origin_users_rows {
        if rows >= 10.0 {
            stars += 1;
        }
    }
    stars.min(3)
}

/// Parse container sizes string "1|2|4|8|16|24|32" and return max size.
fn max_container_size(sizes: &Option<String>) -> u32 {
    match sizes {
        Some(s) => s.split('|').filter_map(|v| v.parse::<u32>().ok()).max().unwrap_or(0),
        None => 0,
    }
}

/// Check container compatibility.
fn container_compatible(ship_max: u32, route_sizes: &Option<String>) -> bool {
    if ship_max == 0 {
        return false;
    }
    max_container_size(route_sizes) <= ship_max
}

/// Estimate round-trip fuel cost for a route.
fn estimate_fuel_cost(distance_gm: Option<f64>, extra_gm: f64, hydrogen_price: f64) -> f64 {
    let distance = distance_gm.unwrap_or(0.0) + extra_gm;
    if distance <= 0.0 {
        return 0.0;
    }
    let scu_consumed = (distance / 100.0) * FUEL_CONSUMPTION_PER_100GM;
    scu_consumed * hydrogen_price * ROUND_TRIP_MULTIPLIER
}

/// Find hydrogen price from commodity list.
fn find_hydrogen_price(commodities: &[Commodity]) -> f64 {
    commodities
        .iter()
        .filter(|c| c.name.to_lowercase() == "hydrogen")
        .filter_map(|c| c.price_sell)
        .find(|&p| p > 0.0)
        .unwrap_or(DEFAULT_HYDROGEN_PRICE)
}

/// Calculate data age in days from Unix timestamp.
fn data_age_days(route: &Route) -> Option<u32> {
    route.date_added.map(|ts| {
        let now = chrono::Utc::now().timestamp() as u64;
        (now.saturating_sub(ts) / 86400) as u32
    })
}

/// Whether a route's origin orbit is a Lagrange/jump point connecting to Pyro.
fn is_lagrange_jump_orbit(orbit_id: Option<u32>) -> bool {
    orbit_id.map(|id| STANTON_LAGRANGE_ORBITS.contains(&id)).unwrap_or(false)
}

/// Get the system name string for a given system ID.
fn system_name_for_id(system_id: u32) -> &'static str {
    match system_id {
        68 => SYSTEM_NAME_STANTON,
        64 => SYSTEM_NAME_PYRO,
        55 => SYSTEM_NAME_NYX,
        _ => SYSTEM_NAME_STANTON,
    }
}

/// Process and rank trade routes for a given cargo capacity and system.
pub fn rank_routes(
    routes: &[Route],
    commodities: &[Commodity],
    cargo_scu: u32,
    ship_max_container: Option<u32>,
    system_id: u32,
) -> Vec<RankedRoute> {
    let hydrogen_price = find_hydrogen_price(commodities);
    let system_name = system_name_for_id(system_id);

    let mut ranked: Vec<RankedRoute> = routes
        .iter()
        .filter(|r| {
            // System filter: same-system filter for Stanton/Nyx,
            // or accept cross-system Stanton→Pyro routes when system_id is Pyro
            let origin_sys = r.origin_star_system_name.as_deref();
            let dest_sys = r.destination_star_system_name.as_deref();
            match system_id {
                64 => {
                    // Pyro: show Pyro→Pyro intra-system AND Stanton→Pyro cross-system routes
                    (origin_sys == Some("Pyro") && dest_sys == Some("Pyro"))
                        || (origin_sys == Some("Stanton") && dest_sys == Some("Pyro"))
                }
                55 => {
                    // Nyx: show Nyx→Nyx routes
                    origin_sys == Some("Nyx") && dest_sys == Some("Nyx")
                }
                _ => {
                    // Stanton (default): intra-system only
                    origin_sys == Some(system_name) && dest_sys == Some(system_name)
                }
            }
        })
        .filter(|r| r.price_margin > MIN_MARGIN_PCT)
        .filter(|r| r.scu_origin.unwrap_or(f64::MAX) >= 1.0)
        .filter(|r| {
            if let Some(max) = ship_max_container {
                container_compatible(max, &r.container_sizes_destination)
            } else {
                true
            }
        })
        .map(|r| {
            let available_scu = r.scu_origin.unwrap_or(f64::MAX) as u32;
            let scu_to_trade = cargo_scu.min(available_scu).max(1);

            let gross_profit = (r.price_destination - r.price_origin) * scu_to_trade as f64;
            let fuel_cost = estimate_fuel_cost(r.distance, 0.0, hydrogen_price);
            let net_profit = gross_profit - fuel_cost;
            let profit_per_scu = if scu_to_trade > 0 {
                net_profit / scu_to_trade as f64
            } else {
                0.0
            };

            let is_player_owned = r.origin_terminal_is_player_owned.unwrap_or(0) == 1;
            let is_at_lagrange = is_lagrange_jump_orbit(r.orbit_origin_id);
            let jump_count = if is_at_lagrange { 1 } else { 0 };

            RankedRoute {
                rank: 0,
                stars: calculate_stars(r, data_age_days(r)),
                commodity: r.commodity_name.clone(),
                commodity_slug: r.commodity_slug.clone(),
                origin: r.terminal_origin_name.clone(),
                destination: r.terminal_destination_name.clone(),
                scu_to_trade,
                buy_price: r.price_origin,
                sell_price: r.price_destination,
                total_profit: net_profit,
                profit_per_scu,
                margin_pct: r.price_margin,
                stock_level: StockLevel::from_status(r.status_destination),
                fuel_cost,
                container_sizes: r.container_sizes_destination.clone().unwrap_or_default(),
                distance_gm: r.distance.unwrap_or(0.0),
                data_age_days: data_age_days(r),
                is_player_owned,
                destination_slug: r.destination_terminal_slug.clone(),
                is_interstellar: is_at_lagrange,
                jump_count,
                destination_system: if is_at_lagrange {
                    Some("Pyro".to_string())
                } else {
                    None
                },
            }
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, route) in ranked.iter_mut().take(MAX_RESULTS).enumerate() {
        route.rank = (i + 1) as u8;
    }

    ranked.into_iter().take(MAX_RESULTS).collect()
}

/// Process cross-system (interstellar) routes: buy in one system, sell in another.
/// Combines routes from origin and destination systems.
pub fn rank_interstellar_routes(
    routes_origin: &[Route],
    routes_dest: &[Route],
    commodities: &[Commodity],
    cargo_scu: u32,
    ship_max_container: Option<u32>,
    _system_id: u32,
) -> Vec<RankedRoute> {
    let hydrogen_price = find_hydrogen_price(commodities);

    // Build a map of commodity_id → best sell price in destination system
    let dest_prices: std::collections::HashMap<u32, &Route> = routes_dest
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, r| {
            acc.entry(r.commodity_id).or_insert(r);
            acc
        });

    let mut ranked: Vec<RankedRoute> = routes_origin
        .iter()
        .filter(|r| {
            // Only origin routes that are in a different system than Stanton
            r.origin_star_system_name.as_deref() != Some(SYSTEM_NAME_STANTON)
        })
        .filter(|r| r.price_margin > MIN_MARGIN_PCT)
        .filter(|r| r.scu_origin.unwrap_or(f64::MAX) >= 1.0)
        .filter(|r| {
            if let Some(max) = ship_max_container {
                container_compatible(max, &r.container_sizes_destination)
            } else {
                true
            }
        })
        .filter_map(|r| {
            let dest_route = dest_prices.get(&r.commodity_id)?;
            let available_scu = r.scu_origin.unwrap_or(f64::MAX) as u32;
            let scu_to_trade = cargo_scu.min(available_scu).max(1);

            let buy_price = r.price_origin;
            let sell_price = dest_route.price_destination;

            let gross_profit = (sell_price - buy_price) * scu_to_trade as f64;
            // Extra fuel for the quantum jump (round trip)
            let extra_fuel_gm = QUANTUM_FUEL_PER_JUMP * 2.0;
            let fuel_cost = estimate_fuel_cost(r.distance, extra_fuel_gm, hydrogen_price);
            let net_profit = gross_profit - fuel_cost;
            let profit_per_scu = if scu_to_trade > 0 {
                net_profit / scu_to_trade as f64
            } else {
                0.0
            };
            let margin_pct = if buy_price > 0.0 {
                ((sell_price - buy_price) / buy_price) * 100.0
            } else {
                0.0
            };
            let is_player_owned = r.origin_terminal_is_player_owned.unwrap_or(0) == 1;
            let dest_sys_name = r.destination_star_system_name.clone().unwrap_or_else(|| "Pyro".to_string());

            Some(RankedRoute {
                rank: 0,
                stars: calculate_stars(r, data_age_days(r)),
                commodity: r.commodity_name.clone(),
                commodity_slug: r.commodity_slug.clone(),
                origin: r.terminal_origin_name.clone(),
                destination: dest_route.terminal_destination_name.clone(),
                scu_to_trade,
                buy_price,
                sell_price,
                total_profit: net_profit,
                profit_per_scu,
                margin_pct,
                stock_level: StockLevel::from_status(dest_route.status_destination),
                fuel_cost,
                container_sizes: dest_route
                    .container_sizes_destination
                    .clone()
                    .unwrap_or_default(),
                distance_gm: r.distance.unwrap_or(0.0),
                data_age_days: data_age_days(r),
                is_player_owned,
                destination_slug: dest_route.destination_terminal_slug.clone(),
                is_interstellar: true,
                jump_count: 1,
                destination_system: Some(dest_sys_name),
            })
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, route) in ranked.iter_mut().take(MAX_RESULTS).enumerate() {
        route.rank = (i + 1) as u8;
    }

    ranked.into_iter().take(MAX_RESULTS).collect()
}

pub fn available_systems() -> Vec<(u32, &'static str)> {
    vec![
        (SYSTEM_ID_STANTON, SYSTEM_NAME_STANTON),
        (64, SYSTEM_NAME_PYRO),
        (55, SYSTEM_NAME_NYX),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commodity(id: u32, name: &str, price_sell: f64) -> Commodity {
        Commodity {
            id,
            parent_id: None,
            name: name.into(),
            code: None,
            kind: None,
            weight_scu: None,
            price_buy: None,
            price_sell: Some(price_sell),
            is_available: Some(1),
            is_available_live: Some(1),
            is_visible: Some(1),
            is_buyable: Some(1),
            is_sellable: Some(1),
            is_illegal: None,
            is_fuel: None,
        }
    }

    fn make_route(price_origin: f64, price_destination: f64, scu_origin: f64, score: f64, system: &str) -> Route {
        Route {
            id: 1,
            commodity_id: 1,
            star_system_origin_id: 68,
            star_system_destination_id: 68,
            planet_origin_id: Some(1),
            planet_destination_id: Some(1),
            orbit_origin_id: Some(1),
            orbit_destination_id: Some(1),
            terminal_origin_id: 1,
            terminal_destination_id: 2,
            faction_origin_id: Some(1),
            faction_destination_id: Some(1),
            code: "TEST".into(),
            price_origin,
            price_destination,
            price_origin_users: None,
            price_destination_users: None,
            price_origin_users_rows: None,
            price_destination_users_rows: None,
            price_margin: ((price_destination - price_origin) / price_origin) * 100.0,
            price_roi: 0.0,
            scu_origin: Some(scu_origin),
            scu_origin_users: None,
            scu_origin_users_rows: None,
            scu_destination: Some(100.0),
            scu_destination_users: None,
            scu_destination_users_rows: None,
            scu_margin: None,
            scu_reachable: Some(scu_origin),
            volatility_origin: None,
            volatility_destination: None,
            status_origin: Some(50),
            status_destination: Some(80),
            investment: 0.0,
            profit: 0.0,
            distance: Some(50.0),
            score: Some(score),
            container_sizes_origin: None,
            container_sizes_destination: Some("1|2|4|8|16".into()),
            has_docking_port_origin: Some(1),
            has_docking_port_destination: Some(1),
            has_freight_elevator_origin: Some(1),
            has_freight_elevator_destination: Some(1),
            has_loading_dock_origin: Some(1),
            has_loading_dock_destination: Some(1),
            has_refuel_origin: Some(1),
            has_refuel_destination: Some(1),
            has_cargo_center_origin: Some(1),
            has_cargo_center_destination: Some(1),
            has_quantum_marker_origin: Some(1),
            has_quantum_marker_destination: Some(1),
            is_monitored_origin: Some(1),
            is_monitored_destination: Some(1),
            is_space_station_origin: Some(1),
            is_space_station_destination: Some(1),
            is_on_ground_origin: Some(0),
            is_on_ground_destination: Some(0),
            commodity_name: "Laranite".into(),
            commodity_code: Some("LAR".into()),
            commodity_slug: Some("laranite".into()),
            origin_star_system_name: Some(system.into()),
            origin_planet_name: Some("Hurston".into()),
            origin_orbit_name: None,
            terminal_origin_name: "HURSTON".into(),
            terminal_origin_code: Some("HUR".into()),
            terminal_origin_slug: Some("hurston".into()),
            origin_terminal_is_player_owned: Some(0),
            origin_faction_name: Some("UEE".into()),
            destination_star_system_name: Some(system.into()),
            destination_planet_name: Some("MicroTech".into()),
            destination_orbit_name: None,
            terminal_destination_name: "ORISON".into(),
            terminal_destination_code: Some("ORI".into()),
            terminal_destination_slug: Some("orison".into()),
            destination_terminal_is_player_owned: Some(0),
            destination_faction_name: Some("UEE".into()),
            game_version_origin: Some("3.24".into()),
            game_version_destination: Some("3.24".into()),
            date_added: Some(chrono::Utc::now().timestamp() as u64 - 86400),
        }
    }

    #[test]
    fn test_rank_routes_high_profit_first() {
        let routes = vec![
            make_route(100.0, 110.0, 96.0, 5.0, SYSTEM_NAME_STANTON),
            make_route(100.0, 150.0, 96.0, 8.0, SYSTEM_NAME_STANTON),
            make_route(100.0, 115.0, 96.0, 3.0, SYSTEM_NAME_STANTON),
        ];
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked = rank_routes(&routes, &commodities, 96, None, SYSTEM_ID_STANTON);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].commodity, "Laranite");
        assert!(ranked[0].total_profit > ranked[1].total_profit);
    }

    #[test]
    fn test_fuel_cost_subtracted() {
        let route = make_route(100.0, 200.0, 96.0, 8.0, SYSTEM_NAME_STANTON);
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked = rank_routes(&[route], &commodities, 96, None, SYSTEM_ID_STANTON);

        assert_eq!(ranked.len(), 1);
        assert!(ranked[0].total_profit < 9600.0);
    }

    #[test]
    fn test_low_margin_filtered_out() {
        let mut route = make_route(100.0, 102.0, 96.0, 5.0, SYSTEM_NAME_STANTON);
        route.price_margin = 2.0;
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked = rank_routes(&[route], &commodities, 96, None, SYSTEM_ID_STANTON);

        assert_eq!(ranked.len(), 0);
    }

    #[test]
    fn test_system_filter() {
        let stanton_route = make_route(100.0, 150.0, 96.0, 8.0, SYSTEM_NAME_STANTON);
        let pyro_route = make_route(100.0, 150.0, 96.0, 8.0, SYSTEM_NAME_PYRO);
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked = rank_routes(
            &[stanton_route, pyro_route],
            &commodities,
            96,
            None,
            SYSTEM_ID_STANTON,
        );

        // Only Stanton routes should appear
        assert_eq!(ranked.len(), 1);
    }
}
