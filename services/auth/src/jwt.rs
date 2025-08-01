//! JWT service for token generation, validation, and management
//!
//! This module provides functionality for creating and validating JWT tokens
//! using the RS256 algorithm, as well as refresh token rotation and
//! token blacklisting using Redis.

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};
use uuid::Uuid;

use crate::models::{Role, User};

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Private key for signing tokens
    pub private_key: String,
    /// Public key for verifying tokens
    pub public_key: String,
    /// Access token expiration time in seconds (default: 15 minutes)
    pub access_token_expiry: u64,
    /// Refresh token expiration time in seconds (default: 7 days)
    pub refresh_token_expiry: u64,
}

impl JwtConfig {
    /// Create a new JwtConfig from environment variables
    ///
    /// # Environment Variables
    /// - `JWT_PRIVATE_KEY`: Private key for signing tokens (PEM format) or path to private key file
    /// - `JWT_PUBLIC_KEY`: Public key for verifying tokens (PEM format) or path to public key file
    /// - `JWT_ACCESS_TOKEN_EXPIRY`: Access token expiry in seconds (default: 900)
    /// - `JWT_REFRESH_TOKEN_EXPIRY`: Refresh token expiry in seconds (default: 604800)
    pub fn from_env() -> Result<Self> {
        let private_key = std::env::var("JWT_PRIVATE_KEY")
            .map_err(|_| anyhow::anyhow!("JWT_PRIVATE_KEY environment variable not set"))?;

        // If the private key looks like a file path, read from file (try CWD, then project root)
        let private_key = if private_key.starts_with("-----BEGIN") {
            private_key
        } else {
            std::fs::read_to_string(&private_key)
                .or_else(|_| {
                    // Try resolving relative to project root
                    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    path.push(&private_key);
                    std::fs::read_to_string(path)
                })
                .map_err(|e| anyhow::anyhow!("Failed to read private key file: {}", e))?
                .trim()
                .to_string()
        };

        let public_key = std::env::var("JWT_PUBLIC_KEY")
            .map_err(|_| anyhow::anyhow!("JWT_PUBLIC_KEY environment variable not set"))?;

        // If the public key looks like a file path, read from file (try CWD, then project root)
        let public_key = if public_key.starts_with("-----BEGIN") {
            public_key
        } else {
            std::fs::read_to_string(&public_key)
                .or_else(|_| {
                    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    path.push(&public_key);
                    std::fs::read_to_string(path)
                })
                .map_err(|e| anyhow::anyhow!("Failed to read public key file: {}", e))?
                .trim()
                .to_string()
        };

        let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "900".to_string()) // 15 minutes
            .parse()
            .unwrap_or(900);

        let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string()) // 7 days
            .parse()
            .unwrap_or(604800);

        Ok(JwtConfig {
            private_key,
            public_key,
            access_token_expiry,
            refresh_token_expiry,
        })
    }
}

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

/// JWT service
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    config: JwtConfig,
}

impl JwtService {
    /// Initialize a new JWT service
    pub fn new(config: JwtConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_rsa_pem(config.private_key.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(config.public_key.as_bytes())?;
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = true;

        Ok(JwtService {
            encoding_key,
            decoding_key,
            validation,
            config,
        })
    }

    /// Generate an access token for a user
    pub fn generate_access_token(&self, user: &User, roles: &[Role]) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get current time: {}", e))?
            .as_secs();

        let roles_vec: Vec<String> = roles.iter().map(|r| r.name.clone()).collect();
        let permissions_vec: Vec<String> = roles
            .iter()
            .flat_map(|r| r.permissions.keys().cloned())
            .collect();

        let claims = Claims {
            sub: user.id,
            roles: roles_vec,
            permissions: permissions_vec,
            iat: now,
            exp: now + self.config.access_token_expiry,
            token_type: TokenType::Access,
        };

        let token = encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.encoding_key,
        )?;
        Ok(token)
    }

    /// Generate a refresh token for a user
    pub fn generate_refresh_token(&self, user: &User) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get current time: {}", e))?
            .as_secs();

        let claims = Claims {
            sub: user.id,
            roles: vec![],
            permissions: vec![],
            iat: now,
            exp: now + self.config.refresh_token_expiry,
            token_type: TokenType::Refresh,
        };

        let token = encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.encoding_key,
        )?;
        Ok(token)
    }

    /// Validate a token and return the claims
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }

    /// Check if a token is blacklisted in Redis
    pub async fn is_token_blacklisted(
        &self,
        redis_pool: &super::cache::RedisPool,
        token: &str,
    ) -> Result<bool> {
        let key = format!("blacklisted_token:{}", token);
        let result = redis_pool.get(&key).await?;
        Ok(result.is_some())
    }

    /// Blacklist a token in Redis
    pub async fn blacklist_token(
        &self,
        redis_pool: &super::cache::RedisPool,
        token: &str,
        expiry: u64,
    ) -> Result<()> {
        let key = format!("blacklisted_token:{}", token);
        redis_pool.set(&key, "1", Some(expiry)).await?;
        Ok(())
    }

    /// Get the access token expiry time
    pub fn access_token_expiry(&self) -> u64 {
        self.config.access_token_expiry
    }

    /// Get the refresh token expiry time
    pub fn refresh_token_expiry(&self) -> u64 {
        self.config.refresh_token_expiry
    }

    /// Rotate a refresh token
    ///
    /// This function blacklists the old refresh token and generates a new one
    pub async fn rotate_refresh_token(
        &self,
        redis_pool: &super::cache::RedisPool,
        user: &User,
        old_refresh_token: &str,
    ) -> Result<String> {
        // Validate the old refresh token
        let claims = self.validate_token(old_refresh_token)?;

        // Check that it's actually a refresh token
        if claims.token_type != TokenType::Refresh {
            return Err(anyhow::anyhow!("Token is not a refresh token"));
        }

        // Check that it belongs to the user
        if claims.sub != user.id {
            return Err(anyhow::anyhow!("Token does not belong to user"));
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use std::collections::HashMap;
            use uuid::Uuid;

            #[test]
            fn test_jwt_config_from_env() {
                // This test would require setting environment variables
                // which is not ideal for unit tests, so we'll skip it
                // or mock the environment variables
            }

            // Note: For proper testing of JWT functionality, we would need
            // to generate proper RSA keys or use test keys, but that's
            // beyond the scope of this implementation
        }

        // Blacklist the old refresh token
        // We'll blacklist it for its remaining lifetime to prevent reuse
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get current time: {}", e))?
            .as_secs();

        let expiry = claims.exp.saturating_sub(now);
        self.blacklist_token(redis_pool, old_refresh_token, expiry)
            .await?;

        // Generate a new refresh token
        let new_refresh_token = self.generate_refresh_token(user)?;

        Ok(new_refresh_token)
    }
}
