//! Session management using Redis

use anyhow::Result;
use tracing::info;
use uuid::Uuid;

use crate::{cache::RedisPool, jwt::JwtService};

/// Session manager for handling user sessions in Redis
#[derive(Clone)]
pub struct SessionManager {
    redis_pool: RedisPool,
    jwt_service: JwtService,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(redis_pool: RedisPool, jwt_service: JwtService) -> Self {
        Self {
            redis_pool,
            jwt_service,
        }
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: Uuid, refresh_token: &str) -> Result<()> {
        info!("Creating session for user: {}", user_id);

        let session_key = format!("session:{}", user_id);
        self.redis_pool
            .set(
                &session_key,
                refresh_token,
                Some(self.jwt_service.refresh_token_expiry()),
            )
            .await?;

        Ok(())
    }

    /// Get a session for a user
    pub async fn get_session(&self, user_id: Uuid) -> Result<Option<String>> {
        info!("Getting session for user: {}", user_id);

        let session_key = format!("session:{}", user_id);
        let refresh_token = self.redis_pool.get(&session_key).await?;

        Ok(refresh_token)
    }

    /// Update an existing session
    pub async fn update_session(&self, user_id: Uuid, refresh_token: &str) -> Result<()> {
        info!("Updating session for user: {}", user_id);

        let session_key = format!("session:{}", user_id);
        self.redis_pool
            .set(
                &session_key,
                refresh_token,
                Some(self.jwt_service.refresh_token_expiry()),
            )
            .await?;

        Ok(())
    }

    /// Delete a session for a user
    pub async fn delete_session(&self, user_id: Uuid) -> Result<()> {
        info!("Deleting session for user: {}", user_id);

        let session_key = format!("session:{}", user_id);
        self.redis_pool.delete(&session_key).await?;

        Ok(())
    }

    /// Delete all sessions for a user (logout from all devices)
    pub async fn delete_all_sessions(&self, user_id: Uuid) -> Result<()> {
        info!("Deleting all sessions for user: {}", user_id);

        // In a more complex implementation, we might store multiple sessions per user
        // For now, we just delete the single session
        self.delete_session(user_id).await?;

        Ok(())
    }

    /// Check if a session exists and is valid
    pub async fn is_session_valid(&self, user_id: Uuid, refresh_token: &str) -> Result<bool> {
        info!("Checking if session is valid for user: {}", user_id);

        let stored_token = self.get_session(user_id).await?;

        match stored_token {
            Some(token) => Ok(token == refresh_token),
            None => Ok(false),
        }
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        info!("Cleaning up expired sessions");

        // Redis automatically expires keys with TTL, so we don't need to do anything here
        // In a more complex implementation, we might want to clean up additional session data
        Ok(0)
    }

    /// Get Redis health status
    pub async fn health_check(&self) -> Result<bool> {
        self.redis_pool.health_check().await
    }
}
