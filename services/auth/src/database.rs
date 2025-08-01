//! Database connection and pooling for the authentication service

use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::{error, info};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl DatabaseConfig {
    /// Create a new DatabaseConfig from environment variables
    ///
    /// # Environment Variables
    /// - `DATABASE_URL`: PostgreSQL connection URL
    /// - `DATABASE_MAX_CONNECTIONS`: Maximum number of connections (default: 10)
    /// - `DATABASE_MIN_CONNECTIONS`: Minimum number of connections (default: 5)
    /// - `DATABASE_CONNECTION_TIMEOUT`: Connection timeout in seconds (default: 30)
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable not set"))?;

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let connection_timeout = std::env::var("DATABASE_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Ok(DatabaseConfig {
            database_url,
            max_connections,
            min_connections,
            connection_timeout,
        })
    }
}

/// Initialize a PostgreSQL connection pool
///
/// # Arguments
/// * `config` - Database configuration
///
/// # Returns
/// * `Result<PgPool>` - PostgreSQL connection pool
pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool> {
    info!("Initializing database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connection_timeout))
        .connect(&config.database_url)
        .await?;

    info!("Database connection pool initialized successfully");
    Ok(pool)
}

/// Check database connectivity
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
/// * `Result<bool>` - True if database is reachable, false otherwise
pub async fn health_check(pool: &PgPool) -> Result<bool> {
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => {
            info!("Database health check successful");
            Ok(true)
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_database_config_from_env() {
        // Set environment variables for testing
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
        }

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.database_url, "postgresql://test:test@localhost/test");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connection_timeout, 30);

        // Clean up
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn test_database_config_from_env_with_custom_values() {
        // Set environment variables for testing
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
            std::env::set_var("DATABASE_MAX_CONNECTIONS", "20");
            std::env::set_var("DATABASE_MIN_CONNECTIONS", "10");
            std::env::set_var("DATABASE_CONNECTION_TIMEOUT", "60");
        }

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.database_url, "postgresql://test:test@localhost/test");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.connection_timeout, 60);

        // Clean up
        unsafe {
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("DATABASE_MAX_CONNECTIONS");
            std::env::remove_var("DATABASE_MIN_CONNECTIONS");
            std::env::remove_var("DATABASE_CONNECTION_TIMEOUT");
        }
    }
}
