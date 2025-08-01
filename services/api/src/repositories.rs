//! Repositories for database operations

use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{CreateUserRequest, SessionResponse, UserResponse};

pub mod media;

/// User repository for database operations
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
    pub async fn create(&self, payload: &CreateUserRequest) -> Result<UserResponse> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (email, provider, roles, settings)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, created_at, updated_at
            "#,
        )
        .bind(&payload.email)
        .bind("local") // provider
        .bind(serde_json::Value::Array(vec![])) // roles as empty array
        .bind(serde_json::Value::Object(serde_json::Map::new())) // settings as empty object
        .fetch_one(&self.pool)
        .await?;

        let user = UserResponse {
            id: row.get("id"),
            username: payload.username.clone(), // Not stored in DB but return for API consistency
            email: row.get("email"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user)
    }

    /// Get all users
    pub async fn get_all(&self) -> Result<Vec<UserResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT id, email, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| UserResponse {
                id: row.get("id"),
                username: String::new(), // Not stored in DB
                email: row.get("email"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(users)
    }

    /// Find a user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserResponse>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = UserResponse {
                    id: row.get("id"),
                    username: String::new(), // Not stored in DB
                    email: row.get("email"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}

/// Session repository for database operations
#[derive(Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    /// Create a new session repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all sessions for a user
    pub async fn get_sessions_by_user_id(&self, user_id: Uuid) -> Result<Vec<SessionResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT token as id, user_id, expires_at, created_at
            FROM sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let sessions = rows
            .into_iter()
            .map(|row| SessionResponse {
                id: row.get("id"),
                user_id: row.get("user_id"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(sessions)
    }

    /// Delete a session by ID
    pub async fn delete_session(&self, session_id: Uuid) -> Result<bool> {
        // Session ID is actually the token in the current schema
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE user_id = $1
            "#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
