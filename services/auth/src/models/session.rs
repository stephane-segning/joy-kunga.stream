//! Session model and related functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Session entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New session creation payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

/// Session update payload
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSession {
    pub expires_at: Option<DateTime<Utc>>,
}
