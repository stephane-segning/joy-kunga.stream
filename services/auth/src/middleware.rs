//! Middleware for JWT token validation and authentication

use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{error, info};

use crate::{AppState, jwt::JwtService, models::User};

/// Extract and validate JWT token from Authorization header
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the Authorization header
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if the header starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract the token
    let token = &auth_header[7..];

    // Validate the token
    let claims = state.jwt_service.validate_token(token).map_err(|e| {
        error!("Failed to validate token: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Check if the token is blacklisted
    let is_blacklisted = state
        .jwt_service
        .is_token_blacklisted(&state.redis_pool, token)
        .await
        .map_err(|e| {
            error!("Failed to check if token is blacklisted: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if is_blacklisted {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add user ID to request extensions for use in handlers
    req.extensions_mut().insert(claims.sub);

    // Continue with the request
    Ok(next.run(req).await)
}
