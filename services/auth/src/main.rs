use anyhow::Result;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod database;
mod models;

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

    // TODO: Start the web server here

    Ok(())
}
