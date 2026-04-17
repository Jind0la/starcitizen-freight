//! UEX API 2.0 client for Freight
//!
//! Base URL: https://api.uexcorp.space/2.0/
//! Auth: Bearer token (optional for public endpoints)
//! Rate limit: 172,800/day or 120/min

use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::models::{ApiResponse, FuelPrice, Route, Terminal};
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
    fuel_prices: RwLock<Option<CacheEntry<Vec<FuelPrice>>>>,
    terminals: RwLock<Option<CacheEntry<Vec<Terminal>>>>,
}

impl Default for ApiCache {
    fn default() -> Self {
        Self {
            routes: RwLock::new(None),
            fuel_prices: RwLock::new(None),
            terminals: RwLock::new(None),
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
    pub fn new(token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Freight/0.1.0 (Star Citizen Cargo Calculator)")
            .build()
            .expect("reqwest client must build");

        Self {
            client,
            base_url: "https://api.uexcorp.space/2.0".to_string(),
            token,
            cache: Arc::new(ApiCache::default()),
        }
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut rb = self.client.get(url);
        if let Some(token) = &self.token {
            rb = rb.header("Authorization", format!("Bearer {token}"));
        }
        rb
    }

    // ------------------------------------------------------------------------
    // Routes
    // ------------------------------------------------------------------------

    /// Fetch all commodities routes from UEX API.
    /// Cached for 30 minutes (matches API cache TTL).
    pub async fn get_routes(&self) -> Result<Vec<Route>, AppError> {
        // Check cache
        {
            let cache = self.cache.routes.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let url = format!("{}/commodities_routes", self.base_url);
        let response = self.build_request(&url).send().await
            .map_err(|_| AppError::ApiUnreachable)?;

        if response.status() == 429 {
            return Err(AppError::RateLimited);
        }

        let parsed: ApiResponse<Vec<Route>> = response.json().await
            .map_err(|_| AppError::InvalidResponse("Failed to parse routes response".into()))?;

        if parsed.status != "ok" {
            return Err(AppError::InvalidResponse(parsed.status));
        }

        // Update cache
        {
            let mut cache = self.cache.routes.write().await;
            *cache = Some(CacheEntry {
                data: parsed.data.clone(),
                expires_at: Instant::now() + Duration::from_secs(30 * 60),
            });
        }

        Ok(parsed.data)
    }

    // ------------------------------------------------------------------------
    // Fuel Prices
    // ------------------------------------------------------------------------

    /// Fetch hydrogen fuel prices (for fuel cost estimation).
    /// Cached for 30 minutes.
    pub async fn get_fuel_prices(&self) -> Result<Vec<FuelPrice>, AppError> {
        // Check cache
        {
            let cache = self.cache.fuel_prices.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let url = format!("{}/fuel_prices", self.base_url);
        let response = self.build_request(&url).send().await
            .map_err(|_| AppError::ApiUnreachable)?;

        if response.status() == 429 {
            return Err(AppError::RateLimited);
        }

        let parsed: ApiResponse<Vec<FuelPrice>> = response.json().await
            .map_err(|_| AppError::InvalidResponse("Failed to parse fuel prices response".into()))?;

        if parsed.status != "ok" {
            return Err(AppError::InvalidResponse(parsed.status));
        }

        // Update cache
        {
            let mut cache = self.cache.fuel_prices.write().await;
            *cache = Some(CacheEntry {
                data: parsed.data.clone(),
                expires_at: Instant::now() + Duration::from_secs(30 * 60),
            });
        }

        Ok(parsed.data)
    }

    // ------------------------------------------------------------------------
    // Terminals
    // ------------------------------------------------------------------------

    /// Fetch terminals for Stanton system.
    /// Cached for 12 hours.
    pub async fn get_terminals(&self) -> Result<Vec<Terminal>, AppError> {
        // Check cache
        {
            let cache = self.cache.terminals.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let url = format!("{}/terminals?id_star_system=1", self.base_url);
        let response = self.build_request(&url).send().await
            .map_err(|_| AppError::ApiUnreachable)?;

        if response.status() == 429 {
            return Err(AppError::RateLimited);
        }

        let parsed: ApiResponse<Vec<Terminal>> = response.json().await
            .map_err(|_| AppError::InvalidResponse("Failed to parse terminals response".into()))?;

        if parsed.status != "ok" {
            return Err(AppError::InvalidResponse(parsed.status));
        }

        // Update cache
        {
            let mut cache = self.cache.terminals.write().await;
            *cache = Some(CacheEntry {
                data: parsed.data.clone(),
                expires_at: Instant::now() + Duration::from_secs(12 * 60 * 60),
            });
        }

        Ok(parsed.data)
    }
}
