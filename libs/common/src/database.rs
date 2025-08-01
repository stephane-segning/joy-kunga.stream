//! Database module for handling PostgreSQL connections and operations
//!
//! This module provides connection pooling, configuration, and health checks
//! for the PostgreSQL database.

use crate::error::{DatabaseError, DatabaseResult};
use sqlx::{PgPool, Pool, Postgres};
use std::env;

/// Database configuration struct
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Create a new DatabaseConfig from environment variables
    pub fn from_env() -> DatabaseResult<Self> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/joy_kunga".to_string()
        });

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        Ok(Self {
            database_url,
            max_connections,
        })
    }
}

/// Initialize a PostgreSQL connection pool
///
/// # Arguments
///
/// * `config` - Database configuration
///
/// # Returns
///
/// * `DatabaseResult<Pool<Postgres>>` - PostgreSQL connection pool or error
pub async fn init_pool(config: &DatabaseConfig) -> DatabaseResult<Pool<Postgres>> {
    let pool = PgPool::connect_with(
        config
            .database_url
            .parse()
            .map_err(|e| DatabaseError::Configuration(format!("Invalid database URL: {}", e)))?,
    )
    .await
    .map_err(DatabaseError::Connection)?;

    Ok(pool)
}

/// Check database connectivity
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// * `DatabaseResult<bool>` - True if connection is successful, false otherwise
pub async fn health_check(pool: &PgPool) -> DatabaseResult<bool> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(DatabaseError::Query)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_from_env() {
        let config = DatabaseConfig::from_env().expect("Failed to create database config");
        assert_eq!(config.max_connections, 5);
        assert_eq!(
            config.database_url,
            "postgresql://postgres:postgres@localhost:5432/joy_kunga"
        );
    }
}
