use anyhow::Result;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod error;
mod middleware;
mod models;
mod repositories;
mod routes;
mod state;

use crate::repositories::media;

use axum::Router;
use common::database::{DatabaseConfig, init_pool};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::{
    repositories::{SessionRepository, UserRepository},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting API service");

    // Initialize database connection pool
    let db_config = DatabaseConfig::from_env()?;
    let pool = init_pool(&db_config).await?;

    // Check database connectivity
    if common::database::health_check(&pool).await? {
        info!("Database connection successful");
    } else {
        anyhow::bail!("Failed to connect to database");
    }

    info!("API service initialized successfully");

    // Initialize repositories
    let user_repository = UserRepository::new(pool.clone());
    let session_repository = SessionRepository::new(pool.clone());
    let media_repository = media::MediaRepository::new(pool.clone());

    let app_state = AppState {
        db_pool: pool,
        user_repository,
        session_repository,
        media_repository,
    };

    // Start the web server
    let app = routes::create_router(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("API service listening on 0.0.0.0:3001");

    axum::serve(listener, app).await?;

    Ok(())
}
