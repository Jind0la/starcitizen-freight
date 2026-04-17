//! Route filtering, ranking, and profit calculation logic.

use crate::models::{RankedRoute, Route, StockLevel};

/// Default hydrogen price (CR per SCU) when API data is unavailable.
const DEFAULT_HYDROGEN_PRICE: f64 = 15.0;

/// Estimated hydrogen SCU consumed per 100 GM of quantum travel.
const FUEL_CONSUMPTION_PER_100GM: f64 = 10.0;

/// Round-trip multiplier for fuel calculation.
const ROUND_TRIP_MULTIPLIER: f64 = 2.0;

/// Minimum price margin (%) to consider a route worth showing.
const MIN_MARGIN_PCT: f64 = 5.0;

/// Maximum routes to return.
const MAX_RESULTS: usize = 3;

/// Star rating thresholds based on score and data freshness.
fn calculate_stars(route: &Route, data_age_days: Option<u32>) -> u8 {
    let mut stars = 1u8;

    // High score adds a star
    if route.score.unwrap_or(0.0) >= 7.0 {
        stars += 1;
    }

        // Recent user trade data adds a star
        if let Some(rows) = route.price_origin_users_rows {
            if rows >= 10.0 {
                stars += 1;
            }
        }

    stars.min(3)
}

/// Estimate round-trip fuel cost for a route.
fn estimate_fuel_cost(distance_gm: Option<f64>, hydrogen_price: f64) -> f64 {
    let distance = distance_gm.unwrap_or(0.0);
    if distance <= 0.0 {
        return 0.0;
    }

    let scu_consumed = (distance / 100.0) * FUEL_CONSUMPTION_PER_100GM;
    scu_consumed * hydrogen_price * ROUND_TRIP_MULTIPLIER
}

/// Find the average hydrogen fuel price from fuel price list.
fn find_hydrogen_price(prices: &[crate::models::FuelPrice]) -> f64 {
    prices
        .iter()
        .filter(|p| p.commodity_name.to_lowercase().contains("hydrogen"))
        .map(|p| p.price_buy)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(DEFAULT_HYDROGEN_PRICE)
}

/// Calculate data age in days from Unix timestamp.
fn data_age_days(route: &Route) -> Option<u32> {
    route.date_added.map(|ts| {
        let now = chrono::Utc::now().timestamp() as u64;
        let diff_secs = now.saturating_sub(ts);
        (diff_secs / 86400) as u32
    })
}

/// Process and rank trade routes for a given cargo capacity.
pub fn rank_routes(
    routes: &[Route],
    fuel_prices: &[crate::models::FuelPrice],
    cargo_scu: u32,
) -> Vec<RankedRoute> {
    let hydrogen_price = find_hydrogen_price(fuel_prices);

    let mut ranked: Vec<RankedRoute> = routes
        .iter()
        .filter(|r| {
            // Filter out negative or zero margin routes
            r.price_margin > MIN_MARGIN_PCT
        })
        .filter(|r| {
            // Filter routes where user's cargo can actually fit
            let available = r.scu_origin.unwrap_or(f64::MAX);
            available >= 1.0
        })
        .filter(|r| {
            // Only Stanton system for now
            r.origin_star_system_name.as_deref() == Some("Stanton")
                && r.destination_star_system_name.as_deref() == Some("Stanton")
        })
        .map(|r| {
            let available_scu = r.scu_origin.unwrap_or(f64::MAX) as u32;
            let scu_to_trade = cargo_scu.min(available_scu).max(1);

            let gross_profit = (r.price_destination - r.price_origin) * scu_to_trade as f64;
            let fuel_cost = estimate_fuel_cost(r.distance, hydrogen_price);
            let net_profit = gross_profit - fuel_cost;
            let profit_per_scu = if scu_to_trade > 0 {
                net_profit / scu_to_trade as f64
            } else {
                0.0
            };

            RankedRoute {
                rank: 0,
                stars: calculate_stars(r, data_age_days(r)),
                commodity: r.commodity_name.clone(),
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
            }
        })
        .collect();

    // Sort by total net profit descending
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_route(
        price_origin: f64,
        price_destination: f64,
        scu_origin: f64,
        score: f64,
    ) -> Route {
        Route {
            id: 1,
            commodity_id: 1,
            star_system_origin_id: 1,
            star_system_destination_id: 1,
            planet_origin_id: Some(1),
            planet_destination_id: Some(1),
            terminal_origin_id: 1,
            terminal_destination_id: 2,
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
            scu_destination: Some(100.0),
            scu_margin: None,
            scu_origin_users: None,
            scu_destination_users: None,
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
            origin_faction_name: Some("UEE".into()),
            destination_star_system_name: Some("Stanton".into()),
            destination_planet_name: Some("MicroTech".into()),
            destination_orbit_name: None,
            terminal_destination_name: "ORISON".into(),
            terminal_destination_code: Some("ORI".into()),
            terminal_destination_slug: Some("orison".into()),
            destination_faction_name: Some("UEE".into()),
            game_version_origin: Some("3.24".into()),
            game_version_destination: Some("3.24".into()),
            date_added: Some(chrono::Utc::now().timestamp() as u64 - 86400),
        }
    }

    #[test]
    fn test_rank_routes_high_profit_first() {
        let routes = vec![
            make_route(100.0, 110.0, 96.0, 5.0), // 10 SCU margin
            make_route(100.0, 150.0, 96.0, 8.0), // 50 SCU margin - better
            make_route(100.0, 115.0, 96.0, 3.0), // 15 SCU margin
        ];
        let fuel_prices: Vec<crate::models::FuelPrice> = vec![];

        let ranked = rank_routes(&routes, &fuel_prices, 96);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].commodity, "Laranite");
        assert!(ranked[0].total_profit > ranked[1].total_profit);
    }

    #[test]
    fn test_fuel_cost_subtracted() {
        let route = make_route(100.0, 200.0, 96.0, 8.0);
        // margin = 100%, scu = 96, gross = 9600
        // fuel = (50gm / 100) * 10 * 15 * 2 = 150
        // net = 9600 - 150 = 9450
        let fuel_prices: Vec<crate::models::FuelPrice> = vec![];

        let ranked = rank_routes(&[route], &fuel_prices, 96);

        assert_eq!(ranked.len(), 1);
        assert!(ranked[0].total_profit < 9600.0); // fuel subtracted
    }

    #[test]
    fn test_low_margin_filtered_out() {
        let mut route = make_route(100.0, 102.0, 96.0, 5.0);
        route.price_margin = 2.0; // below 5% threshold
        let fuel_prices: Vec<crate::models::FuelPrice> = vec![];

        let ranked = rank_routes(&[route], &fuel_prices, 96);

        assert_eq!(ranked.len(), 0);
    }
}
