//! Authentication service routes

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use oauth2::TokenResponse;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    AppState,
    jwt::Claims,
    models::{LoginCredentials, NewUser, User},
    oauth::OAuthProvider,
    rate_limiter::RateLimiter,
    repositories::UserRepository,
    validation,
};

/// Response for token generation
#[derive(Serialize)]
pub struct TokenGenerationResponse {
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
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Request for user registration
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Request for user login
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

/// Request for OAuth authorization
#[derive(Deserialize)]
pub struct OAuthAuthRequest {
    pub provider: String,
}

/// Request for OAuth callback
#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
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
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/oauth/authorize", post(oauth_authorize))
        .route("/auth/oauth/callback", post(oauth_callback))
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

/// User registration endpoint
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("Registration attempt for user: {}", payload.username);

    // Validate input
    validation::validate_username(&payload.username).map_err(|e| AuthError::BadRequest(e))?;
    validation::validate_email(&payload.email).map_err(|e| AuthError::BadRequest(e))?;
    validation::validate_password(&payload.password).map_err(|e| AuthError::BadRequest(e))?;

    // Check if user already exists
    if let Some(_) = state
        .user_repository
        .find_by_username_or_email(&payload.username)
        .await
        .map_err(|e| {
            error!("Failed to check existing user by username: {}", e);
            AuthError::InternalServerError
        })?
    {
        return Err(AuthError::BadRequest("Username already exists".to_string()));
    }

    if let Some(_) = state
        .user_repository
        .find_by_username_or_email(&payload.email)
        .await
        .map_err(|e| {
            error!("Failed to check existing user by email: {}", e);
            AuthError::InternalServerError
        })?
    {
        return Err(AuthError::BadRequest("Email already exists".to_string()));
    }

    // Create new user
    let new_user = NewUser {
        username: payload.username,
        email: payload.email,
        password_hash: payload.password,
    };

    let user = state.user_repository.create(&new_user).await.map_err(|e| {
        error!("Failed to create user: {}", e);
        AuthError::InternalServerError
    })?;

    let response = serde_json::json!({
        "user_id": user.id.to_string(),
        "message": "User registered successfully"
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// User login endpoint
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("Login attempt for user: {}", payload.username_or_email);

    // Validate input
    if payload.username_or_email.is_empty() {
        return Err(AuthError::BadRequest(
            "Username or email is required".to_string(),
        ));
    }

    if payload.password.is_empty() {
        return Err(AuthError::BadRequest("Password is required".to_string()));
    }

    // Find user by username or email
    let user = state
        .user_repository
        .find_by_username_or_email(&payload.username_or_email)
        .await
        .map_err(|e| {
            error!("Failed to find user: {}", e);
            AuthError::InternalServerError
        })?
        .ok_or(AuthError::Unauthorized)?;

    // Verify password
    let is_valid = state
        .user_repository
        .verify_password(&user, &payload.password)
        .await
        .map_err(|e| {
            error!("Failed to verify password: {}", e);
            AuthError::InternalServerError
        })?;

    if !is_valid {
        return Err(AuthError::Unauthorized);
    }

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

    let response = TokenGenerationResponse {
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

    let response = TokenRefreshResponse {
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

/// OAuth authorization endpoint
pub async fn oauth_authorize(
    State(state): State<AppState>,
    Json(payload): Json<OAuthAuthRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!(
        "OAuth authorization request for provider: {}",
        payload.provider
    );

    // Determine the OAuth provider
    let provider = match payload.provider.as_str() {
        "google" => OAuthProvider::Google,
        "apple" => OAuthProvider::Apple,
        _ => {
            return Err(AuthError::BadRequest(
                "Unsupported OAuth provider".to_string(),
            ));
        }
    };

    // Get the appropriate OAuth client
    let oauth_client = match &provider {
        OAuthProvider::Google => state.google_oauth_client.as_ref(),
        OAuthProvider::Apple => state.apple_oauth_client.as_ref(),
    };

    let oauth_client = match oauth_client {
        Some(client) => client,
        None => return Err(AuthError::InternalServerError),
    };

    // Generate authorization URL
    let scopes = match &provider {
        OAuthProvider::Google => &["email", "profile"],
        OAuthProvider::Apple => &["email", "name"],
    };

    let (auth_url, csrf_token, pkce_verifier) =
        oauth_client.generate_auth_url(scopes).map_err(|e| {
            error!("Failed to generate authorization URL: {}", e);
            AuthError::InternalServerError
        })?;

    // Store session data in Redis
    let session_key = format!("oauth_session:{}", csrf_token.secret());
    let session = crate::oauth::OAuthSession::new(
        csrf_token.secret().clone(),
        pkce_verifier.secret().clone(),
        provider.clone(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                error!("Failed to get current time: {}", e);
                AuthError::InternalServerError
            })?
            .as_secs(),
    );

    let session_json = serde_json::to_string(&session).map_err(|e| {
        error!("Failed to serialize OAuth session: {}", e);
        AuthError::InternalServerError
    })?;

    state
        .redis_pool
        .set(
            &session_key,
            &session_json,
            Some(300), // 5 minutes expiry
        )
        .await
        .map_err(|e| {
            error!("Failed to store OAuth session in Redis: {}", e);
            AuthError::InternalServerError
        })?;

    let response = serde_json::json!({
        "auth_url": auth_url,
        "provider": provider.as_str()
    });

    Ok((StatusCode::OK, Json(response)))
}

/// OAuth callback endpoint
pub async fn oauth_callback(
    State(state): State<AppState>,
    Json(payload): Json<OAuthCallbackRequest>,
) -> Result<impl IntoResponse, AuthError> {
    info!("OAuth callback request");

    if let Some(error) = payload.error {
        return Err(AuthError::BadRequest(format!("OAuth error: {}", error)));
    }

    let code = match payload.code {
        Some(code) => code,
        None => {
            return Err(AuthError::BadRequest(
                "Missing authorization code".to_string(),
            ));
        }
    };

    let state_param = match payload.state {
        Some(state_param) => state_param,
        None => return Err(AuthError::BadRequest("Missing state parameter".to_string())),
    };

    // Retrieve session data from Redis
    let session_key = format!("oauth_session:{}", state_param);
    let session_json = state
        .redis_pool
        .get(&session_key)
        .await
        .map_err(|e| {
            error!("Failed to retrieve OAuth session from Redis: {}", e);
            AuthError::InternalServerError
        })?
        .ok_or_else(|| AuthError::BadRequest("Invalid or expired OAuth session".to_string()))?;

    let session: crate::oauth::OAuthSession = serde_json::from_str(&session_json).map_err(|e| {
        error!("Failed to deserialize OAuth session: {}", e);
        AuthError::InternalServerError
    })?;

    // Remove session from Redis (one-time use)
    state.redis_pool.delete(&session_key).await.map_err(|e| {
        error!("Failed to remove OAuth session from Redis: {}", e);
        AuthError::InternalServerError
    })?;

    // Get the appropriate OAuth client
    let oauth_client = match &session.provider {
        crate::oauth::OAuthProvider::Google => state.google_oauth_client.as_ref(),
        crate::oauth::OAuthProvider::Apple => state.apple_oauth_client.as_ref(),
    };

    let oauth_client = match oauth_client {
        Some(client) => client,
        None => return Err(AuthError::InternalServerError),
    };

    // Exchange authorization code for access token
    let pkce_verifier = oauth2::PkceCodeVerifier::new(session.pkce_verifier);
    let token_response = oauth_client
        .exchange_code(code, pkce_verifier)
        .await
        .map_err(|e| {
            error!("Failed to exchange authorization code: {}", e);
            AuthError::InternalServerError
        })?;

    // Get user profile information
    let access_token = token_response.access_token().secret();
    let user_profile = oauth_client
        .get_user_profile(access_token)
        .await
        .map_err(|e| {
            error!("Failed to get user profile: {}", e);
            AuthError::InternalServerError
        })?;

    // Check if user already exists
    let existing_user = state
        .user_repository
        .find_by_username_or_email(&user_profile.email)
        .await
        .map_err(|e| {
            error!("Failed to check existing user: {}", e);
            AuthError::InternalServerError
        })?;

    // Create or link user account
    let user = if let Some(user) = existing_user {
        // User already exists, update OAuth information
        info!("Linking existing user {} with OAuth provider", user.id);
        user
    } else {
        // Create new user
        info!("Creating new user with OAuth provider");
        let new_user = crate::models::NewUser {
            username: format!("{}_{}", session.provider.as_str(), user_profile.id),
            email: user_profile.email.clone(),
            password_hash: "".to_string(), // OAuth users don't have passwords
        };

        state.user_repository.create(&new_user).await.map_err(|e| {
            error!("Failed to create user: {}", e);
            AuthError::InternalServerError
        })?
    };

    // Generate JWT tokens
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

    let response = serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": state.jwt_service.access_token_expiry(),
        "user_id": user.id.to_string(),
        "email": user.email,
        "provider": session.provider.as_str()
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Custom error type for authentication errors
#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    BadRequest(String),
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
