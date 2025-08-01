//! Custom error types for the common library
//!
//! This module defines application-specific error types that can be used
//! throughout the application.

use sqlx::Error as SqlxError;
use thiserror::Error;

/// Custom error type for database operations
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Error occurred during database connection
    #[error("Database connection error: {0}")]
    Connection(#[source] SqlxError),

    /// Error occurred during database query execution
    #[error("Database query error: {0}")]
    Query(#[source] SqlxError),

    /// Error occurred during database migration
    #[error("Database migration error: {0}")]
    Migration(String),

    /// Configuration error
    #[error("Database configuration error: {0}")]
    Configuration(String),
}

/// Type alias for Result with DatabaseError
pub type DatabaseResult<T> = Result<T, DatabaseError>;
