//! Route filtering, ranking, and profit calculation logic.
//!
//! Fuel estimation:
//! - Hydrogen consumption: ~10 SCU H2 per 100 GM quantum travel
//! - Hydrogen price: fetched from API or default 400 CR/SCU
//! - Jump point traversal: +30 SCU H2 per jump
//!
//! Profit formula per route:
//!   gross_profit = (sell_price - buy_price) × min(scu_available, user_scu)
//!   fuel_cost = estimate_quantum_fuel(distance, is_interstellar)
//!   net_profit = gross_profit - fuel_cost

use crate::models::{
    Commodity, LoopRoute, RankedRoute, Route, StockLevel, AVAILABLE_SYSTEMS, SYSTEM_ID_PYRO,
    SYSTEM_ID_STANTON, SYSTEM_NAME_PYRO, SYSTEM_NAME_STANTON,
};

/// Hydrogen consumption: SCU H2 per 100 GM quantum travel.
/// Real in-game is ~10 SCU/100GM for most ships.
const SCU_H2_PER_100GM: f64 = 10.0;

/// Additional hydrogen SCU consumed per quantum jump point traversal.
const SCU_H2_PER_JUMP: f64 = 30.0;

/// Round-trip multiplier for fuel (go there and back).
const ROUND_TRIP_MULTIPLIER: f64 = 2.0;

/// Minimum quantum fuel cost to cover QT spool etc (aUEC).
const MIN_FUEL_COST: f64 = 200.0;

/// Minimum price margin (%) to consider a route.
const MIN_MARGIN_PCT: f64 = 5.0;

/// Maximum routes to return in results.
const MAX_RESULTS: usize = 25;

/// Known Lagrange orbit IDs in Stanton that are at jump gates to Pyro.
/// These routes appear as Stanton→Pyro cross-system routes.
const STANTON_JUMP_GATE_ORBITS: &[u32] = &[
    361, // Terra Gateway
    339, // microTech Lagrange Point 1
    326, // ArcCorp Lagrange Point 1
    333, // Crusader Lagrange Point 5
    398, // Stanton Gateway (Pyro side)
];

/// Star rating thresholds.
fn calculate_stars(route: &Route) -> u8 {
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
    sizes
        .as_ref()
        .and_then(|s| {
            s.split('|')
                .filter_map(|v| v.parse::<u32>().ok())
                .max()
        })
        .unwrap_or(0)
}

/// Check container compatibility with ship's max container size.
fn container_compatible(ship_max: u32, route_sizes: &Option<String>) -> bool {
    if ship_max == 0 {
        return false;
    }
    max_container_size(route_sizes) <= ship_max
}

/// Estimate round-trip quantum fuel cost for a route in aUEC.
/// Uses hydrogen price from commodity list.
fn estimate_fuel_cost(
    distance_gm: Option<f64>,
    hydrogen_price: f64,
    is_interstellar: bool,
    jump_count: u8,
) -> f64 {
    let distance = distance_gm.unwrap_or(0.0);

    // Intra-system quantum travel fuel
    let scu_travel = (distance / 100.0) * SCU_H2_PER_100GM * ROUND_TRIP_MULTIPLIER;

    // Extra fuel for jump point traversals (Stanton↔Pyro)
    let scu_jumps = jump_count as f64 * SCU_H2_PER_JUMP * ROUND_TRIP_MULTIPLIER;

    let total_scu = scu_travel + scu_jumps;
    let cost = total_scu * hydrogen_price;

    // Minimum fuel cost even for very short routes (QT spool overhead)
    cost.max(MIN_FUEL_COST)
}

/// Find hydrogen commodity price from the commodity list.
/// Falls back to default if not found.
fn find_hydrogen_price(commodities: &[Commodity]) -> f64 {
    commodities
        .iter()
        .filter(|c| c.name.to_lowercase() == "hydrogen")
        .filter_map(|c| c.price_sell)
        .find(|&p| p > 0.0)
        .unwrap_or(400.0)
}

/// Calculate data age in days from Unix timestamp.
fn data_age_days(route: &Route) -> Option<u32> {
    route.date_added.map(|ts| {
        let now = chrono::Utc::now().timestamp() as u64;
        (now.saturating_sub(ts) / 86400) as u32
    })
}

/// Whether a route is cross-system (origin and destination in different systems).
fn is_cross_system(route: &Route) -> bool {
    route.star_system_origin_id != route.star_system_destination_id
}

/// Whether the origin is a known jump gate orbit (connecting Stanton↔Pyro).
fn is_jump_gate_route(route: &Route) -> bool {
    route.orbit_origin_id.map(|id| STANTON_JUMP_GATE_ORBITS.contains(&id)).unwrap_or(false)
}

/// Get jump count for a route.
fn get_jump_count(route: &Route) -> u8 {
    if is_cross_system(route) {
        // Stanton↔Pyro is 1 jump through the gate
        1
    } else {
        0
    }
}

/// Determine destination system name from route data.
fn destination_system_name(route: &Route) -> Option<String> {
    if is_cross_system(route) {
        route.destination_star_system_name.clone()
    } else {
        None
    }
}

// ─── Route classification ─────────────────────────────────────────────────────

/// Tab/view filter for route results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTab {
    /// All routes (intra-system + interstellar)
    All,
    /// Only intra-system routes (origin and destination in same system)
    IntraSystem,
    /// Only cross-system routes (Stanton↔Pyro etc)
    Interstellar,
}

impl RouteTab {
    pub fn from_str(s: &str) -> Self {
        match s {
            "intra" | "intra-system" => RouteTab::IntraSystem,
            "interstellar" | "cross-system" => RouteTab::Interstellar,
            _ => RouteTab::All,
        }
    }
}

/// Filter routes by tab.
fn filter_by_tab<'a>(routes: &'a [Route], tab: RouteTab) -> Vec<&'a Route> {
    routes
        .iter()
        .filter(|r| {
            match tab {
                RouteTab::All => true,
                RouteTab::IntraSystem => !is_cross_system(r),
                RouteTab::Interstellar => is_cross_system(r),
            }
        })
        .collect()
}

// ─── Main ranking function ────────────────────────────────────────────────────

/// Process and rank trade routes for a given cargo capacity and system.
/// `system_id` filters origin system. Use 0 for all systems.
/// `tab` filters by intra/interstellar.
/// `min_margin` overrides MIN_MARGIN_PCT if provided.
pub fn rank_routes(
    routes: &[Route],
    commodities: &[Commodity],
    cargo_scu: u32,
    ship_max_container: Option<u32>,
    system_id: u32,
    tab: RouteTab,
    min_margin: Option<f64>,
) -> Vec<RankedRoute> {
    let hydrogen_price = find_hydrogen_price(commodities);
    let min_margin = min_margin.unwrap_or(MIN_MARGIN_PCT);

    let all_filtered = routes
        .iter()
        .filter(|r| {
            // System filter
            if system_id != 0 {
                let origin_match = r.star_system_origin_id == system_id;
                if !origin_match {
                    return false;
                }
            }

            // Tab filter
            match tab {
                RouteTab::All => true,
                RouteTab::IntraSystem => !is_cross_system(r),
                RouteTab::Interstellar => is_cross_system(r),
            }
        })
        .filter(|r| r.price_margin > min_margin)
        .filter(|r| r.scu_origin.unwrap_or(f64::MAX) >= 1.0)
        .filter(|r| {
            if let Some(max) = ship_max_container {
                container_compatible(max, &r.container_sizes_destination)
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    let mut ranked: Vec<RankedRoute> = all_filtered
        .iter()
        .filter_map(|r| {
            let available_scu = r.scu_origin.unwrap_or(f64::MAX) as u32;
            let scu_to_trade = cargo_scu.min(available_scu).max(1);

            let gross_profit = (r.price_destination - r.price_origin) * scu_to_trade as f64;

            let jump_count = get_jump_count(r);
            let fuel_cost =
                estimate_fuel_cost(r.distance, hydrogen_price, is_cross_system(r), jump_count);
            let net_profit = gross_profit - fuel_cost;

            let profit_per_scu = if scu_to_trade > 0 {
                net_profit / scu_to_trade as f64
            } else {
                0.0
            };

            let is_player_owned = r.origin_terminal_is_player_owned.unwrap_or(0) == 1;
            let dest_slug = r.destination_terminal_slug.clone();
            let is_interstellar = is_cross_system(r);
            let dest_sys = destination_system_name(r);

            Some(RankedRoute {
                rank: 0,
                stars: calculate_stars(r),
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
                container_sizes: r
                    .container_sizes_destination
                    .clone()
                    .unwrap_or_default(),
                distance_gm: r.distance.unwrap_or(0.0),
                data_age_days: data_age_days(r),
                is_player_owned,
                destination_slug: dest_slug,
                is_interstellar,
                jump_count,
                destination_system: dest_sys,
            })
        })
        .collect();

    // Sort by net profit descending
    ranked.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Assign ranks
    for (i, route) in ranked.iter_mut().take(MAX_RESULTS).enumerate() {
        route.rank = (i + 1) as u8;
    }

    ranked.into_iter().take(MAX_RESULTS).collect()
}

/// Compute cross-system (interstellar) route profit.
/// This combines a buy route in one system with a sell route in another system
/// to compute the actual profit of Stanton→Pyro (or Pyro→Stanton) runs.
pub fn compute_interstellar_profit(
    routes_all: &[Route],
    commodities: &[Commodity],
    cargo_scu: u32,
    hydrogen_price: f64,
) -> Vec<RankedRoute> {
    // Find Stanton→Pyro routes (origin Stanton, dest Pyro)
    let stanton_routes: Vec<_> = routes_all
        .iter()
        .filter(|r| {
            r.star_system_origin_id == SYSTEM_ID_STANTON
                && r.star_system_destination_id == SYSTEM_ID_PYRO
        })
        .collect();

    // Find Pyro→Stanton routes
    let pyro_routes: Vec<_> = routes_all
        .iter()
        .filter(|r| {
            r.star_system_origin_id == SYSTEM_ID_PYRO
                && r.star_system_destination_id == SYSTEM_ID_STANTON
        })
        .collect();

    // Build sell-price lookup per commodity in each destination system
    // For Stanton→Pyro: buy in Stanton (stanton_routes), sell at Pyro terminals
    // We need Pyro sell prices per commodity
    let mut pyro_sell_prices: std::collections::HashMap<u32, &Route> =
        std::collections::HashMap::new();
    for r in routes_all.iter().filter(|r| r.star_system_destination_id == SYSTEM_ID_PYRO) {
        pyro_sell_prices
            .entry(r.commodity_id)
            .or_insert_with(|| r);
    }

    // For Pyro→Stanton: sell in Stanton
    let mut stanton_sell_prices: std::collections::HashMap<u32, &Route> =
        std::collections::HashMap::new();
    for r in routes_all
        .iter()
        .filter(|r| r.star_system_destination_id == SYSTEM_ID_STANTON)
    {
        stanton_sell_prices
            .entry(r.commodity_id)
            .or_insert_with(|| r);
    }

    let mut results = Vec::new();

    // Stanton → Pyro (buy Stanton, sell Pyro)
    for buy_route in &stanton_routes {
        if buy_route.price_margin <= MIN_MARGIN_PCT {
            continue;
        }
        if buy_route.scu_origin.unwrap_or(f64::MAX) < 1.0 {
            continue;
        }

        // Find sell price in Pyro for same commodity
        let Some(sell_route) = pyro_sell_prices.get(&buy_route.commodity_id) else {
            continue;
        };

        let available_scu = buy_route.scu_origin.unwrap_or(f64::MAX) as u32;
        let scu_to_trade = cargo_scu.min(available_scu).max(1);

        let buy_price = buy_route.price_origin;
        let sell_price = sell_route.price_destination;
        let gross_profit = (sell_price - buy_price) * scu_to_trade as f64;

        // 1 jump Stanton→Pyro, round trip
        let fuel_cost = estimate_fuel_cost(
            buy_route.distance,
            hydrogen_price,
            true,
            1,
        );
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

        results.push(RankedRoute {
            rank: 0,
            stars: calculate_stars(buy_route),
            commodity: buy_route.commodity_name.clone(),
            commodity_slug: buy_route.commodity_slug.clone(),
            origin: buy_route.terminal_origin_name.clone(),
            destination: sell_route.terminal_destination_name.clone(),
            scu_to_trade,
            buy_price,
            sell_price,
            total_profit: net_profit,
            profit_per_scu,
            margin_pct,
            stock_level: StockLevel::from_status(sell_route.status_destination),
            fuel_cost,
            container_sizes: sell_route
                .container_sizes_destination
                .clone()
                .unwrap_or_default(),
            distance_gm: buy_route.distance.unwrap_or(0.0),
            data_age_days: data_age_days(buy_route),
            is_player_owned: buy_route.origin_terminal_is_player_owned.unwrap_or(0) == 1,
            destination_slug: sell_route.destination_terminal_slug.clone(),
            is_interstellar: true,
            jump_count: 1,
            destination_system: Some("Pyro".to_string()),
        });
    }

    // Pyro → Stanton (buy Pyro, sell Stanton)
    for buy_route in &pyro_routes {
        if buy_route.price_margin <= MIN_MARGIN_PCT {
            continue;
        }
        if buy_route.scu_origin.unwrap_or(f64::MAX) < 1.0 {
            continue;
        }

        let Some(sell_route) = stanton_sell_prices.get(&buy_route.commodity_id) else {
            continue;
        };

        let available_scu = buy_route.scu_origin.unwrap_or(f64::MAX) as u32;
        let scu_to_trade = cargo_scu.min(available_scu).max(1);

        let buy_price = buy_route.price_origin;
        let sell_price = sell_route.price_destination;
        let gross_profit = (sell_price - buy_price) * scu_to_trade as f64;

        // 1 jump Pyro→Stanton, round trip
        let fuel_cost = estimate_fuel_cost(
            buy_route.distance,
            hydrogen_price,
            true,
            1,
        );
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

        results.push(RankedRoute {
            rank: 0,
            stars: calculate_stars(buy_route),
            commodity: buy_route.commodity_name.clone(),
            commodity_slug: buy_route.commodity_slug.clone(),
            origin: buy_route.terminal_origin_name.clone(),
            destination: sell_route.terminal_destination_name.clone(),
            scu_to_trade,
            buy_price,
            sell_price,
            total_profit: net_profit,
            profit_per_scu,
            margin_pct,
            stock_level: StockLevel::from_status(sell_route.status_destination),
            fuel_cost,
            container_sizes: sell_route
                .container_sizes_destination
                .clone()
                .unwrap_or_default(),
            distance_gm: buy_route.distance.unwrap_or(0.0),
            data_age_days: data_age_days(buy_route),
            is_player_owned: buy_route.origin_terminal_is_player_owned.unwrap_or(0) == 1,
            destination_slug: sell_route.destination_terminal_slug.clone(),
            is_interstellar: true,
            jump_count: 1,
            destination_system: Some("Stanton".to_string()),
        });
    }

    // Sort and rank
    results.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, route) in results.iter_mut().take(MAX_RESULTS).enumerate() {
        route.rank = (i + 1) as u8;
    }

    results.into_iter().take(MAX_RESULTS).collect()
}

// ─── Loop Routes (A→B→A round trips) ─────────────────────────────────────────

/// Compute all valid A→B→A round-trip loops within a single star system.
/// For each terminal A, finds a commodity to sell A→B and a (possibly different)
/// commodity to buy B→A, then returns to A with a full cargo hold.
/// Both legs share the same cargo capacity.
pub fn compute_loop_routes(
    routes: &[Route],
    commodities: &[Commodity],
    cargo_scu: u32,
    ship_max_container: Option<u32>,
    system_id: u32,
) -> Vec<LoopRoute> {
    let hydrogen_price = find_hydrogen_price(commodities);

    // Group intra-system routes by (origin, destination) terminal pair.
    // outbound: origin=A, destination=B  (sell at B)
    // return:   origin=B, destination=A  (sell at A)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct TerminalPair(u32, u32);

    // outbound routes: A→B (we buy at A, sell at B)
    let mut outbound_by_pair: std::collections::HashMap<
        TerminalPair,
        Vec<&Route>,
    > = std::collections::HashMap::new();

    // return routes: B→A (we buy at B, sell at A)
    let mut return_by_pair: std::collections::HashMap<
        TerminalPair,
        Vec<&Route>,
    > = std::collections::HashMap::new();

    for r in routes.iter().filter(|r| {
        // Intra-system only for now
        r.star_system_origin_id == r.star_system_destination_id
            && (system_id == 0 || r.star_system_origin_id == system_id)
    }) {
        if r.price_margin <= MIN_MARGIN_PCT {
            continue;
        }
        if r.scu_origin.unwrap_or(f64::MAX) < 1.0 {
            continue;
        }
        if let Some(max) = ship_max_container {
            if !container_compatible(max, &r.container_sizes_destination) {
                continue;
            }
        }

        let pair = TerminalPair(r.terminal_origin_id, r.terminal_destination_id);
        outbound_by_pair
            .entry(pair)
            .or_insert_with(Vec::new)
            .push(r);

        // return leg is reversed: origin=destination, destination=origin
        let return_pair = TerminalPair(r.terminal_destination_id, r.terminal_origin_id);
        return_by_pair
            .entry(return_pair)
            .or_insert_with(Vec::new)
            .push(r);
    }

    let mut loops = Vec::new();

    // Iterate over each terminal pair and try to build a loop
    for (pair, outbound_routes) in &outbound_by_pair {
        let TerminalPair(term_a, term_b) = *pair;
        let Some(return_routes) = return_by_pair.get(pair) else {
            continue;
        };

        // Try the best outbound route with the best return route
        // We use the highest-margin outbound route first
        let Some(out) = outbound_routes
            .iter()
            .max_by(|a, b| {
                a.price_margin
                    .partial_cmp(&b.price_margin)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        else {
            continue;
        };

        let Some(ret) = return_routes
            .iter()
            .max_by(|a, b| {
                a.price_margin
                    .partial_cmp(&b.price_margin)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        else {
            continue;
        };

        let scu_to_trade = cargo_scu.min(out.scu_origin.unwrap_or(f64::MAX) as u32).max(1);

        // ── Leg 1: A → B ──────────────────────────────────────────────────────
        let gross_profit_1 = (out.price_destination - out.price_origin) * scu_to_trade as f64;
        let jump_count = get_jump_count(out);
        let is_interstellar = is_cross_system(out);
        let fuel_cost = estimate_fuel_cost(
            out.distance,
            hydrogen_price,
            is_interstellar,
            jump_count,
        );
        // Round-trip fuel covers both legs
        let fuel_per_leg = fuel_cost / 2.0;

        let net_profit_1 = gross_profit_1 - fuel_per_leg;
        let margin_pct_1 = out.price_margin;

        // ── Leg 2: B → A ──────────────────────────────────────────────────────
        let gross_profit_2 =
            (ret.price_destination - ret.price_origin) * scu_to_trade as f64;
        let net_profit_2 = gross_profit_2 - fuel_per_leg;
        let margin_pct_2 = ret.price_margin;

        let total_profit = net_profit_1 + net_profit_2;
        let profit_per_scu = if scu_to_trade > 0 {
            total_profit / scu_to_trade as f64
        } else {
            0.0
        };

        let data_age = data_age_days(out).unwrap_or(0).max(data_age_days(ret).unwrap_or(0));
        let is_player_owned = out.origin_terminal_is_player_owned.unwrap_or(0) == 1;

        loops.push(LoopRoute {
            rank: 0,
            stars: calculate_stars(out).max(calculate_stars(ret)),
            commodity_leg1: out.commodity_name.clone(),
            commodity_leg2: ret.commodity_name.clone(),
            origin: out.terminal_origin_name.clone(),
            destination: out.terminal_destination_name.clone(),
            scu_to_trade,
            buy_price_leg1: out.price_origin,
            sell_price_leg1: out.price_destination,
            buy_price_leg2: ret.price_origin,
            sell_price_leg2: ret.price_destination,
            profit_leg1: net_profit_1,
            profit_leg2: net_profit_2,
            total_profit,
            profit_per_scu,
            margin_pct_leg1: margin_pct_1,
            margin_pct_leg2: margin_pct_2,
            margin_pct: (margin_pct_1 + margin_pct_2) / 2.0,
            stock_level: StockLevel::from_status(out.status_destination),
            fuel_cost,
            container_sizes: out
                .container_sizes_destination
                .clone()
                .unwrap_or_default(),
            distance_gm: out.distance.unwrap_or(0.0),
            data_age_days: Some(data_age),
            is_player_owned,
            is_interstellar,
            jump_count,
            destination_system: None,
        });
    }

    // Sort by total profit descending
    loops.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Assign ranks
    for (i, loop_) in loops.iter_mut().take(MAX_RESULTS).enumerate() {
        loop_.rank = (i + 1) as u8;
    }

    loops.into_iter().take(MAX_RESULTS).collect()
}

// ─── Available systems ───────────────────────────────────────────────────────

pub fn available_systems() -> Vec<(u32, &'static str)> {
    AVAILABLE_SYSTEMS.to_vec()
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

    fn make_route(
        price_origin: f64,
        price_destination: f64,
        scu_origin: f64,
        score: f64,
        system_origin: u32,
        system_dest: u32,
    ) -> Route {
        Route {
            id: 1,
            commodity_id: 1,
            star_system_origin_id: system_origin,
            star_system_destination_id: system_dest,
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
            origin_star_system_name: Some("Stanton".into()),
            origin_planet_name: Some("Hurston".into()),
            origin_orbit_name: None,
            terminal_origin_name: "HURSTON".into(),
            terminal_origin_code: Some("HUR".into()),
            terminal_origin_slug: Some("hurston".into()),
            origin_terminal_is_player_owned: Some(0),
            origin_faction_name: Some("UEE".into()),
            destination_star_system_name: Some("Stanton".into()),
            destination_planet_name: Some("MicroTech".into()),
            destination_orbit_name: None,
            terminal_destination_name: "ORISON".into(),
            destination_terminal_code: Some("ORI".into()),
            destination_terminal_slug: Some("orison".into()),
            destination_terminal_is_player_owned: Some(0),
            destination_faction_name: Some("UEE".into()),
            game_version_origin: Some("4.7".into()),
            game_version_destination: Some("4.7".into()),
            date_added: Some(chrono::Utc::now().timestamp() as u64 - 86400),
        }
    }

    #[test]
    fn test_rank_routes_high_profit_first() {
        let routes = vec![
            make_route(100.0, 110.0, 96.0, 5.0, 68, 68),
            make_route(100.0, 150.0, 96.0, 8.0, 68, 68),
            make_route(100.0, 115.0, 96.0, 3.0, 68, 68),
        ];
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked =
            rank_routes(&routes, &commodities, 96, None, 0, RouteTab::IntraSystem, None);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].commodity, "Laranite");
        assert!(ranked[0].total_profit > ranked[1].total_profit);
    }

    #[test]
    fn test_fuel_cost_subtracted() {
        let route = make_route(100.0, 200.0, 96.0, 8.0, 68, 68);
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked =
            rank_routes(&routes, &commodities, 96, None, 0, RouteTab::IntraSystem, None);

        assert_eq!(ranked.len(), 1);
        assert!(ranked[0].total_profit < 9600.0); // fuel is subtracted
    }

    #[test]
    fn test_low_margin_filtered_out() {
        let mut route = make_route(100.0, 102.0, 96.0, 5.0, 68, 68);
        route.price_margin = 2.0;
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked =
            rank_routes(&[route], &commodities, 96, None, 0, RouteTab::IntraSystem, None);

        assert_eq!(ranked.len(), 0);
    }

    #[test]
    fn test_intra_system_filter() {
        let stanton = make_route(100.0, 150.0, 96.0, 8.0, 68, 68);
        let pyro = make_route(100.0, 150.0, 96.0, 8.0, 64, 64);
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        let ranked =
            rank_routes(&[stanton, pyro], &commodities, 96, None, 68, RouteTab::IntraSystem, None);

        assert_eq!(ranked.len(), 1);
    }

    #[test]
    fn test_interstellar_filter() {
        let stanton_to_pyro =
            make_route(100.0, 150.0, 96.0, 8.0, 68, 64); // Stanton→Pyro
        let pyro_to_stanton =
            make_route(100.0, 150.0, 96.0, 8.0, 64, 68); // Pyro→Stanton
        let intra_stanton = make_route(100.0, 150.0, 96.0, 8.0, 68, 68);
        let commodities = vec![make_commodity(41, "Hydrogen", 800.0)];

        // All routes, interstellar tab
        let ranked = rank_routes(
            &[stanton_to_pyro, pyro_to_stanton, intra_stanton],
            &commodities,
            96,
            None,
            0,
            RouteTab::Interstellar,
            None,
        );

        assert_eq!(ranked.len(), 2); // Both cross-system
        assert!(ranked.iter().all(|r| r.is_interstellar));
    }

    #[test]
    fn test_is_cross_system() {
        let stanton_intra = make_route(100.0, 150.0, 96.0, 8.0, 68, 68);
        let stanton_pyro = make_route(100.0, 150.0, 96.0, 8.0, 68, 64);
        let pyro_stanton = make_route(100.0, 150.0, 96.0, 8.0, 64, 68);

        assert!(!is_cross_system(&stanton_intra));
        assert!(is_cross_system(&stanton_pyro));
        assert!(is_cross_system(&pyro_stanton));
    }
}
