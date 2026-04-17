//! Error types for the Freight application.

use axum::{response::IntoResponse, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("UEX API token required. Set UEX_API_TOKEN in your .env file (get one at https://uexcorp.space/api/apps).")]
    AuthRequired,

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

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::AuthRequired => StatusCode::UNAUTHORIZED,
            AppError::ApiUnreachable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::InvalidResponse(_) => StatusCode::BAD_GATEWAY,
            AppError::NoRoutesFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::HttpError(_) => StatusCode::BAD_GATEWAY,
        };
        let body = serde_json::json!({ "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}

impl AppError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, AppError::ApiUnreachable | AppError::RateLimited)
    }
}
