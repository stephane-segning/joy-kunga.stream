//! Common library for the Joy Kunga application
//!
//! This crate provides shared functionality used across different services
//! in the Joy Kunga application, including database connectivity, error
//! handling, and other common utilities.

pub mod cache;
pub mod database;
pub mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

/// Example usage of the database module
///
/// ```rust,no_run
/// use common::database::{DatabaseConfig, init_pool, health_check};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DatabaseConfig::from_env()?;
///     let pool = init_pool(&config).await?;
///     let is_healthy = health_check(&pool).await?;
///     println!("Database health check: {}", is_healthy);
///     Ok(())
/// }
/// ```
pub fn example_usage() {}
