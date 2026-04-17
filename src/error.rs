//! Error types for the Freight application.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cannot reach UEX API. Check your internet connection.")]
    ApiUnreachable,

    #[error("UEX API rate limit reached. Please wait a moment.")]
    RateLimited,

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("No profitable routes found for {0} SCU. Try a smaller cargo hold or check back later.")]
    NoRoutesFound(u32),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

impl AppError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, AppError::ApiUnreachable | AppError::RateLimited)
    }
}
