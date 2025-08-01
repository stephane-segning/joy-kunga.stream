//! Custom error types for the API service

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Custom error type for the API service
#[derive(Error, Debug)]
pub enum ApiError {
    /// Unauthorized access
    #[error("Unauthorized")]
    Unauthorized,

    /// Bad request with message
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error
    #[error("Internal server error")]
    InternalServerError,

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] common::error::DatabaseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            ApiError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Type alias for API results
pub type ApiResult<T> = Result<T, ApiError>;
