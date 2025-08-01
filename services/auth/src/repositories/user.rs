//! User repository for database operations

use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use sqlx::{PgPool, Row};
use tracing::info;
use uuid::Uuid;

use crate::models::{LoginCredentials, NewUser, User};

/// User repository
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create(&self, new_user: &NewUser) -> Result<User> {
        info!("Creating new user: {}", new_user.username);

        // Hash the password
        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(new_user.password_hash.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(&new_user.username)
        .bind(&new_user.email)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;

        let user = User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user)
    }

    /// Find a user by username or email
    pub async fn find_by_username_or_email(&self, username_or_email: &str) -> Result<Option<User>> {
        info!("Finding user by username or email: {}", username_or_email);

        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE username = $1 OR email = $1
            "#,
        )
        .bind(username_or_email)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Verify a user's password
    pub async fn verify_password(&self, user: &User, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        let argon2 = Argon2::default();
        let result = argon2.verify_password(password.as_bytes(), &parsed_hash);

        Ok(result.is_ok())
    }

    /// Find a user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        info!("Finding user by ID: {}", id);

        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
