//! Authentication service routes

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{AppState, jwt::Claims, models::User};

/// Response for token generation
#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Request for token refresh
#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Response for token refresh
#[derive(Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Request for user login
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response for user login
#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub message: String,
}

/// Create the router for the authentication service
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/logout", post(logout))
        .with_state(state)
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "auth-service"
    }))
}

/// User login endpoint
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("Login attempt for user: {}", payload.username);

    // TODO: Implement actual user authentication
    // This is just a placeholder implementation

    // In a real implementation, we would:
    // 1. Validate the username and password against the database
    // 2. If valid, generate JWT tokens
    // 3. Store session in Redis

    // For now, we'll create a mock user
    let user = User {
        id: uuid::Uuid::new_v4(),
        username: payload.username.clone(),
        email: format!("{}@example.com", payload.username),
        password_hash: "mock_hash".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Generate tokens
    let access_token = state
        .jwt_service
        .generate_access_token(&user, &[])
        .map_err(|e| {
            error!("Failed to generate access token: {}", e);
            AuthError::InternalServerError
        })?;

    let refresh_token = state
        .jwt_service
        .generate_refresh_token(&user)
        .map_err(|e| {
            error!("Failed to generate refresh token: {}", e);
            AuthError::InternalServerError
        })?;

    // Store session in Redis
    // In a real implementation, we would store more session data
    let session_key = format!("session:{}", user.id);
    state
        .redis_pool
        .set(
            &session_key,
            &refresh_token,
            Some(state.jwt_service.refresh_token_expiry()),
        )
        .await
        .map_err(|e| {
            error!("Failed to store session in Redis: {}", e);
            AuthError::InternalServerError
        })?;

    let response = TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.access_token_expiry(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Refresh token endpoint
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("Token refresh request");

    // Validate the refresh token
    let claims = state
        .jwt_service
        .validate_token(&payload.refresh_token)
        .map_err(|_| AuthError::Unauthorized)?;

    // Check that it's actually a refresh token
    if claims.token_type != crate::jwt::TokenType::Refresh {
        return Err(AuthError::Unauthorized);
    }

    // Check if the token is blacklisted
    let is_blacklisted = state
        .jwt_service
        .is_token_blacklisted(&state.redis_pool, &payload.refresh_token)
        .await
        .map_err(|e| {
            error!("Failed to check if token is blacklisted: {}", e);
            AuthError::InternalServerError
        })?;

    if is_blacklisted {
        return Err(AuthError::Unauthorized);
    }

    // Create a mock user for demonstration
    // In a real implementation, we would fetch the user from the database
    let user = User {
        id: claims.sub,
        username: "mock_user".to_string(),
        email: "mock@example.com".to_string(),
        password_hash: "mock_hash".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Generate a new access token
    let access_token = state
        .jwt_service
        .generate_access_token(&user, &[])
        .map_err(|e| {
            error!("Failed to generate access token: {}", e);
            AuthError::InternalServerError
        })?;

    // Rotate the refresh token
    let new_refresh_token = state
        .jwt_service
        .rotate_refresh_token(&state.redis_pool, &user, &payload.refresh_token)
        .await
        .map_err(|e| {
            error!("Failed to rotate refresh token: {}", e);
            AuthError::InternalServerError
        })?;

    let response = RefreshTokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.access_token_expiry(),
    };

    // Update session in Redis
    let session_key = format!("session:{}", user.id);
    state
        .redis_pool
        .set(
            &session_key,
            &new_refresh_token,
            Some(state.jwt_service.refresh_token_expiry()),
        )
        .await
        .map_err(|e| {
            error!("Failed to update session in Redis: {}", e);
            AuthError::InternalServerError
        })?;

    Ok((StatusCode::OK, Json(response)))
}

/// Logout endpoint
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("Logout request");

    // Validate the refresh token
    let claims = state
        .jwt_service
        .validate_token(&payload.refresh_token)
        .map_err(|_| AuthError::Unauthorized)?;

    // Check that it's actually a refresh token
    if claims.token_type != crate::jwt::TokenType::Refresh {
        return Err(AuthError::Unauthorized);
    }

    // Blacklist the refresh token
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| {
            error!("Failed to get current time: {}", e);
            AuthError::InternalServerError
        })?
        .as_secs();

    let expiry = claims.exp.saturating_sub(now);
    state
        .jwt_service
        .blacklist_token(&state.redis_pool, &payload.refresh_token, expiry)
        .await
        .map_err(|e| {
            error!("Failed to blacklist token: {}", e);
            AuthError::InternalServerError
        })?;

    // Remove session from Redis
    let session_key = format!("session:{}", claims.sub);
    state.redis_pool.delete(&session_key).await.map_err(|e| {
        error!("Failed to remove session from Redis: {}", e);
        AuthError::InternalServerError
    })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "Logged out successfully"})),
    ))
}

/// Custom error type for authentication errors
#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
