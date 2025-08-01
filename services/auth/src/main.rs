use anyhow::Result;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod cache;
mod database;
mod jwt;
mod middleware;
mod models;
mod routes;

use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::{cache::RedisPool, jwt::JwtService};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: RedisPool,
    pub jwt_service: JwtService,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting authentication service");

    // Initialize database connection pool
    let db_config = database::DatabaseConfig::from_env()?;
    let pool = database::init_pool(&db_config).await?;

    // Check database connectivity
    if database::health_check(&pool).await? {
        info!("Database connection successful");
    } else {
        anyhow::bail!("Failed to connect to database");
    }

    info!("Authentication service initialized successfully");

    // Initialize JWT service
    let jwt_config = crate::jwt::JwtConfig::from_env()?;
    let jwt_service = crate::jwt::JwtService::new(jwt_config)?;

    // Initialize Redis connection pool
    let redis_config = cache::RedisConfig::from_env()?;
    let redis_pool = cache::RedisPool::new(&redis_config).await?;

    let app_state = AppState {
        db_pool: pool,
        redis_pool,
        jwt_service,
    };

    // Start the web server
    let app = routes::create_router(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Authentication service listening on 0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
