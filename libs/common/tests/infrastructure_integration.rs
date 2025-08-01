//! Integration tests for the infrastructure components
//!
//! These tests verify that the PostgreSQL database and Redis cache
//! are properly configured and accessible from the application.

use common::{
    cache::{RedisConfig, RedisPool},
    database::{DatabaseConfig, health_check, init_pool},
};
use sqlx::Row;

/// Test that verifies both PostgreSQL and Redis are accessible
/// and can perform basic operations
#[tokio::test]
async fn test_infrastructure_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize PostgreSQL connection pool
    let db_config = DatabaseConfig::from_env()?;
    let pool = init_pool(&db_config).await?;

    // Verify PostgreSQL connectivity
    assert!(health_check(&pool).await?, "Database health check failed");

    // Perform a simple query to test database connectivity
    let row = sqlx::query("SELECT 1 as result").fetch_one(&pool).await?;

    let result: i32 = row.get("result");
    assert_eq!(result, 1, "PostgreSQL simple query test failed");

    // Initialize Redis client
    let redis_config = RedisConfig::from_env()?;
    let redis_pool = RedisPool::new(&redis_config).await?;

    // Verify Redis connectivity
    assert!(
        redis_pool.health_check().await?,
        "Redis health check failed"
    );

    // Perform SET/GET operation on Redis
    let test_key = "integration_test_key";
    let test_value = "integration_test_value";

    // Set a key-value pair with TTL
    redis_pool.set(test_key, test_value, Some(10)).await?;

    // Get the value back
    let retrieved_value = redis_pool.get(test_key).await?;
    assert_eq!(
        retrieved_value,
        Some(test_value.to_string()),
        "Redis SET/GET test failed"
    );

    // Clean up - delete the key
    redis_pool.delete(test_key).await?;

    // Verify the key is deleted
    let retrieved_value = redis_pool.get(test_key).await?;
    assert_eq!(retrieved_value, None, "Redis delete operation failed");

    Ok(())
}
