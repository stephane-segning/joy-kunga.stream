//! Authentication middleware for JWT token validation

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: Uuid,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// Issued at time
    pub iat: u64,
    /// Expiration time
    pub exp: u64,
    /// Token type (access or refresh)
    pub token_type: TokenType,
}

/// Token type enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TokenType {
    /// Access token
    Access,
    /// Refresh token
    Refresh,
}

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Public key for verifying tokens
    pub public_key: String,
    /// Access token expiration time in seconds (default: 15 minutes)
    pub access_token_expiry: u64,
    /// Refresh token expiration time in seconds (default: 7 days)
    pub refresh_token_expiry: u64,
}

impl JwtConfig {
    /// Create a new JwtConfig from environment variables
    pub fn from_env() -> Result<Self, String> {
        let public_key = env::var("JWT_PUBLIC_KEY")
            .map_err(|_| "JWT_PUBLIC_KEY environment variable not set".to_string())?;

        // If the public key looks like a file path, read from file (try CWD, then project root)
        let public_key = if public_key.starts_with("-----BEGIN") {
            public_key
        } else {
            std::fs::read_to_string(&public_key)
                .or_else(|_| {
                    // Try resolving relative to project root
                    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    path.push(&public_key);
                    std::fs::read_to_string(path)
                })
                .map_err(|e| format!("Failed to read public key file: {}", e))?
                .trim()
                .to_string()
        };

        let access_token_expiry = env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "900".to_string()) // 15 minutes
            .parse()
            .unwrap_or(900);

        let refresh_token_expiry = env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string()) // 7 days
            .parse()
            .unwrap_or(604800);

        Ok(JwtConfig {
            public_key,
            access_token_expiry,
            refresh_token_expiry,
        })
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract the Authorization header
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    // Check if it's a Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?;

    // Load JWT configuration
    let jwt_config = JwtConfig::from_env().map_err(|e| {
        error!("Failed to load JWT config: {}", e);
        ApiError::InternalServerError
    })?;

    // Create decoding key and validation
    let decoding_key =
        DecodingKey::from_rsa_pem(jwt_config.public_key.as_bytes()).map_err(|e| {
            error!("Failed to create decoding key: {}", e);
            ApiError::InternalServerError
        })?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.validate_exp = true;

    // Validate the token
    let token_data =
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            error!("Failed to validate token: {}", e);
            ApiError::Unauthorized
        })?;

    // Create authenticated user from claims
    let user = AuthUser {
        id: token_data.claims.sub,
        roles: token_data.claims.roles,
        permissions: token_data.claims.permissions,
    };

    // Insert the user into the request extensions
    req.extensions_mut().insert(user);

    // Call the next service
    let response = next.run(req).await;

    Ok(response)
}

/// Extract the authenticated user from the request extensions
pub fn get_current_user<B>(req: &Request<B>) -> Option<AuthUser> {
    req.extensions().get::<AuthUser>().cloned()
}
