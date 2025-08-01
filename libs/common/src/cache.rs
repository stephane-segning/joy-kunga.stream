//! Redis cache module for the Joy Kunga application
//!
//! This module provides functionality for connecting to Redis and performing
//! basic cache operations like get and set with TTL support.

use anyhow::Result;
use redis::{AsyncCommands, Client};
use tracing::info;

/// Configuration for Redis connection
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
}

impl RedisConfig {
    /// Create a new RedisConfig from environment variables
    ///
    /// # Environment Variables
    /// - `REDIS_URL`: Redis connection URL (default: "redis://localhost:6379")
    /// - `REDIS_MAX_CONNECTIONS`: Maximum number of connections (default: 10)
    pub fn from_env() -> Result<Self> {
        let url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let max_connections = std::env::var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        Ok(RedisConfig {
            url,
            max_connections,
        })
    }
}

/// Redis connection pool
pub struct RedisPool {
    client: Client,
}

impl RedisPool {
    /// Initialize a new Redis connection pool
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let client = Client::open(config.url.clone())?;
        info!("Redis client initialized with URL: {}", config.url);
        Ok(RedisPool { client })
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        let conn = self.client.get_multiplexed_async_connection().await?;
        Ok(conn)
    }

    /// Set a key-value pair in Redis with optional TTL
    pub async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let mut conn = self.get_connection().await?;

        if let Some(ttl) = ttl_seconds {
            let _: () = conn.set_ex(key, value, ttl).await?;
        } else {
            let _: () = conn.set(key, value).await?;
        }

        Ok(())
    }

    /// Get a value from Redis by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    /// Delete a key from Redis
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: u64 = conn.del(key).await?;
        Ok(())
    }

    /// Check if Redis is reachable
    pub async fn health_check(&self) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(pong == "PONG")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_connection() -> Result<()> {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
        };

        let pool = RedisPool::new(&config).await?;
        assert!(pool.health_check().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_set_get_delete() -> Result<()> {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
        };

        let pool = RedisPool::new(&config).await?;

        // Test set and get
        let key = "test_key";
        let value = "test_value";
        pool.set(key, value, Some(5)).await?; // Set with 5 seconds TTL

        let retrieved = pool.get(key).await?;
        assert_eq!(retrieved, Some(value.to_string()));

        // Test delete
        pool.delete(key).await?;
        let retrieved = pool.get(key).await?;
        assert_eq!(retrieved, None);

        Ok(())
    }
}
