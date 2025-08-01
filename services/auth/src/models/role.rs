//! Role model and related functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

/// Role entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub permissions: HashMap<String, bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New role creation payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRole {
    pub name: String,
    pub permissions: HashMap<String, bool>,
}

/// Role update payload
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub permissions: Option<HashMap<String, bool>>,
}

/// User role association
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}
