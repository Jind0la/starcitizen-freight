//! UEX API 2.0 client for Freight
//!
//! Base URL: https://api.uexcorp.space/2.0/
//! Auth: Bearer token REQUIRED for /commodities_routes
//! Rate limit: 172,800/day or 120/min (with auth)

use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::models::{ApiResponse, Commodity, Route, Terminal};
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
    terminals: RwLock<Option<CacheEntry<Vec<Terminal>>>>,
}

impl Default for ApiCache {
    fn default() -> Self {
        Self {
            routes: RwLock::new(None),
            commodities: RwLock::new(None),
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
            .timeout(Duration::from_secs(20))
            .user_agent("Freight/0.2.0 (Star Citizen Cargo Calculator)")
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
    // Commodities (needed for hydrogen price lookup)
    // ------------------------------------------------------------------------

    /// Fetch all commodities. No auth required.
    /// Cached for 1 hour.
    pub async fn get_commodities(&self) -> Result<Vec<Commodity>, AppError> {
        {
            let cache = self.cache.commodities.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let url = format!("{}/commodities", self.base_url);
        let response = self.build_request(&url).send().await
            .map_err(|_| AppError::ApiUnreachable)?;

        if response.status() == 429 {
            return Err(AppError::RateLimited);
        }

        let parsed: ApiResponse<Vec<Commodity>> = response.json().await
            .map_err(|_| AppError::InvalidResponse("Failed to parse commodities response".into()))?;

        if parsed.status != "ok" {
            return Err(AppError::InvalidResponse(parsed.status));
        }

        let data = parsed.data;

        {
            let mut cache = self.cache.commodities.write().await;
            *cache = Some(CacheEntry {
                data: data.clone(),
                expires_at: Instant::now() + Duration::from_secs(60 * 60),
            });
        }

        Ok(data)
    }

    // ------------------------------------------------------------------------
    // Routes
    // ------------------------------------------------------------------------

    /// Fetch all commodity routes from UEX API.
    /// Requires auth — fetches routes per-commodity (API requires at least one filter param).
    /// Cached for 30 minutes.
    pub async fn get_routes(&self) -> Result<Vec<Route>, AppError> {
        {
            let cache = self.cache.routes.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let token = self.token.as_ref()
            .ok_or(AppError::AuthRequired)?;

        // Step 1: Get all commodities to enumerate IDs
        let commodities = self.get_commodities().await?;
        let commodity_ids: Vec<u32> = commodities.iter().map(|c| c.id).collect();
        let num_commodities = commodity_ids.len();

        // Step 2: Fetch routes for each commodity
        let mut all_routes = Vec::new();
        let client = &self.client;
        let base_url = &self.base_url;

        // Use semaphore to limit concurrent requests (avoid overwhelming the API)
        let sem = Arc::new(tokio::sync::Semaphore::new(5));
        let mut handles = Vec::new();

        for cid in commodity_ids {
            let permit = sem.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let token = token.clone();
            let base_url = base_url.clone();

            let handle = tokio::spawn(async move {
                let url = format!("{}/commodities_routes/?id_commodity={}", base_url, cid);
                let response = client.get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .timeout(Duration::from_secs(15))
                    .send().await;

                drop(permit);

                match response {
                    Ok(resp) if resp.status() == 200 => {
                        let parsed: ApiResponse<Vec<Route>> = resp.json().await.ok()?;
                        if parsed.status == "ok" {
                            return Some(parsed.data);
                        }
                    }
                    _ => {}
                }
                None
            });
            handles.push(handle);
        }

        // Collect results
        for handle in handles {
            if let Ok(Some(routes)) = handle.await {
                all_routes.extend(routes);
            }
        }

        tracing::info!("Fetched {} total routes for {} commodities",
            all_routes.len(), num_commodities);

        {
            let mut cache = self.cache.routes.write().await;
            *cache = Some(CacheEntry {
                data: all_routes.clone(),
                expires_at: Instant::now() + Duration::from_secs(30 * 60),
            });
        }

        Ok(all_routes)
    }

    // ------------------------------------------------------------------------
    // Terminals
    // ------------------------------------------------------------------------

    /// Fetch terminals. Auth required.
    /// Cached for 12 hours.
    pub async fn get_terminals(&self) -> Result<Vec<Terminal>, AppError> {
        {
            let cache = self.cache.terminals.read().await;
            if let Some(entry) = &*cache {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        let token = self.token.as_ref()
            .ok_or(AppError::AuthRequired)?;

        let url = format!("{}/terminals", self.base_url);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(15))
            .send().await
            .map_err(|_| AppError::ApiUnreachable)?;

        if response.status() == 429 {
            return Err(AppError::RateLimited);
        }

        let parsed: ApiResponse<Vec<Terminal>> = response.json().await
            .map_err(|_| AppError::InvalidResponse("Failed to parse terminals response".into()))?;

        if parsed.status != "ok" {
            return Err(AppError::InvalidResponse(parsed.status));
        }

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
