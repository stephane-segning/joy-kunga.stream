//! Application state shared across handlers

use sqlx::PgPool;

use crate::repositories::{SessionRepository, UserRepository, media::MediaRepository};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub user_repository: UserRepository,
    pub session_repository: SessionRepository,
    pub media_repository: MediaRepository,
}
