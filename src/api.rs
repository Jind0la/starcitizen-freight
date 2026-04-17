//! UEX API 2.0 client for Freight
//!
//! Base URL: https://api.uexcorp.space/2.0/
//! Auth: Bearer token via UEX_API_TOKEN env var (required for routes).
//! Rate limit: 172,800/day or 120/min with auth.

use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::models::{ApiResponse, Commodity, FuelEntry, Route, StarSystem};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache entry with TTL
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Shared cache for API responses
struct ApiCache {
    routes: RwLock<Option<CacheEntry<Vec<Route>>>>,
    commodities: RwLock<Option<CacheEntry<Vec<Commodity>>>>,
    fuel_prices: RwLock<Option<CacheEntry<Vec<FuelEntry>>>>,
    star_systems: RwLock<Option<CacheEntry<Vec<StarSystem>>>>,
}

impl Default for ApiCache {
    fn default() -> Self {
        Self {
            routes: RwLock::new(None),
            commodities: RwLock::new(None),
            fuel_prices: RwLock::new(None),
            star_systems: RwLock::new(None),
        }
    }
}

/// UEX API client
#[derive(Clone)]
pub struct UexClient {
    client: Client,
    base_url: String,
    token: Option<String>,
    cache: Arc<ApiCache>,
}

impl UexClient {
    /// Create a new UEX API client.
    /// If `token` is None, reads UEX_API_TOKEN from the environment.
    pub fn new(token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");

        Self {
            client,
            base_url: "https://api.uexcorp.space/2.0".to_string(),
            token: token.or_else(|| std::env::var("UEX_API_TOKEN").ok()),
            cache: Arc::new(ApiCache::default()),
        }
    }

    /// Build request URL
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Fetch JSON from a path, handling the {status, data} envelope.
    async fn fetch<T: for<'de> serde::Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, AppError> {
        let url = self.url(path);
        let mut req = self.client.get(&url).header("User-Agent", "Freight/1.0");

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }

        let resp = req.send().await?;
        let status = resp.status();

        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AppError::AuthRequired);
        }
        if status.as_u16() == 429 {
            return Err(AppError::RateLimited);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::ApiError(format!(
                "API error {}: {}",
                status.as_u16(),
                text
            )));
        }

        // All UEX API responses follow {status: "ok", data: ...}
        #[derive(serde::Deserialize)]
        struct Envelope<T> {
            status: String,
            data: T,
        }
        let envelope: Envelope<T> = resp.json().await?;
        if envelope.status != "ok" {
            return Err(AppError::ApiError(format!("API status: {}", envelope.status)));
        }
        Ok(envelope.data)
    }

    // ─── Public API Methods ─────────────────────────────────────────────────

    /// Fetch all available star systems (no auth required).
    pub async fn get_star_systems(&self) -> Result<Vec<StarSystem>, AppError> {
        // Check cache first (1 day TTL)
        {
            let cache = self.cache.star_systems.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    tracing::debug!("star_systems: cache hit");
                    return Ok(entry.data.clone());
                }
            }
        }

        let systems: Vec<StarSystem> = self.fetch("/star_systems").await?;

        {
            let mut cache = self.cache.star_systems.write().await;
            *cache = Some(CacheEntry {
                data: systems.clone(),
                expires_at: Instant::now() + Duration::from_secs(86400), // 1 day
            });
        }

        Ok(systems)
    }

    /// Fetch hydrogen fuel prices (no auth required).
    /// Commodity ID 104 = Hydrogen Fuel.
    pub async fn get_fuel_prices(&self) -> Result<Vec<FuelEntry>, AppError> {
        // Check cache first (30 min TTL)
        {
            let cache = self.cache.fuel_prices.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    tracing::debug!("fuel_prices: cache hit");
                    return Ok(entry.data.clone());
                }
            }
        }

        // Fetch hydrogen fuel prices (id_commodity=104)
        let prices: Vec<FuelEntry> = self
            .fetch("/fuel_prices?id_commodity=104")
            .await
            .inspect_err(|e| tracing::warn!("fuel_prices fetch failed: {e}"))
            .unwrap_or_default();

        {
            let mut cache = self.cache.fuel_prices.write().await;
            *cache = Some(CacheEntry {
                data: prices.clone(),
                expires_at: Instant::now() + Duration::from_secs(1800), // 30 min
            });
        }

        Ok(prices)
    }

    /// Fetch all commodities (no auth required).
    pub async fn get_commodities(&self) -> Result<Vec<Commodity>, AppError> {
        // Check cache first (30 min TTL)
        {
            let cache = self.cache.commodities.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    tracing::debug!("commodities: cache hit");
                    return Ok(entry.data.clone());
                }
            }
        }

        let commodities: Vec<Commodity> = self.fetch("/commodities").await?;

        {
            let mut cache = self.cache.commodities.write().await;
            *cache = Some(CacheEntry {
                data: commodities.clone(),
                expires_at: Instant::now() + Duration::from_secs(1800), // 30 min
            });
        }

        Ok(commodities)
    }

    /// Fetch ALL commodity routes by iterating over all commodity IDs.
    /// This is needed because /commodities_routes requires at least one
    /// commodity filter and there's no "get all routes" endpoint.
    ///
    /// We fetch commodities first (cached), then fan out parallel requests
    /// for each commodity's routes, merging them together.
    pub async fn get_routes(&self) -> Result<Vec<Route>, AppError> {
        // Check cache first (30 min TTL)
        {
            let cache = self.cache.routes.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    tracing::debug!("routes: cache hit");
                    return Ok(entry.data.clone());
                }
            }
        }

        // Fetch commodities to get IDs
        let commodities = self.get_commodities().await?;
        let commodity_ids: Vec<u32> = commodities.iter().map(|c| c.id).collect();
        tracing::info!(
            "fetching routes for {} commodities (parallel)",
            commodity_ids.len()
        );

        // Fan out: query routes for each commodity ID in parallel (max 20 concurrent)
        let client = self.clone();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(20));

        let futures: Vec<_> = commodity_ids
            .iter()
            .map(|&id| {
                let c = client.clone();
                let sem = semaphore.clone();
                async move {
                    let _permit = sem.acquire().await.expect("semaphore closed");
                    let routes: Vec<Route> = c
                        .fetch(&format!("/commodities_routes?id_commodity={id}"))
                        .await
                        .unwrap_or_default();
                    routes
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut all_routes = Vec::new();
        for routes in results {
            all_routes.extend(routes);
        }

        // Deduplicate by route ID
        let mut seen = std::collections::HashSet::new();
        all_routes.retain(|r| seen.insert(r.id));

        tracing::info!("total unique routes fetched: {}", all_routes.len());

        {
            let mut cache = self.cache.routes.write().await;
            *cache = Some(CacheEntry {
                data: all_routes.clone(),
                expires_at: Instant::now() + Duration::from_secs(1800), // 30 min
            });
        }

        Ok(all_routes)
    }

    /// Find the average hydrogen fuel price across all terminals.
    /// Returns price in CR per SCU.
    pub async fn average_hydrogen_price(&self) -> f64 {
        let prices = self.get_fuel_prices().await.unwrap_or_default();

        let hydrogen_prices: Vec<f64> = prices
            .iter()
            .filter(|e| e.commodity_id == 104)
            .filter_map(|e| {
                let p = e.effective_price();
                if p > 0.0 {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        if hydrogen_prices.is_empty() {
            tracing::warn!("no hydrogen prices found, using default 400 CR/SCU");
            return 400.0;
        }

        let avg = hydrogen_prices.iter().sum::<f64>() / hydrogen_prices.len() as f64;
        tracing::debug!("average hydrogen price: {} CR/SCU", avg);
        avg
    }
}
