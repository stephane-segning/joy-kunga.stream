use anyhow::Result;
use aws_config::BehaviorVersion;
use sqlx::PgPool;
use std::env;
use tracing::{Level, error, info};
use tracing_subscriber::EnvFilter;

mod database;
mod metadata_extractor;
mod models;
mod s3_poller;
mod thumbnail_generator;

use database::Database;
use s3_poller::S3Poller;
use thumbnail_generator::ThumbnailGenerator;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::INFO)
        .init();

    info!("Starting media ingestion service");

    // Initialize AWS S3 client
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    // Get configuration from environment variables
    let bucket_name = env::var("MEDIA_BUCKET_NAME").unwrap_or_else(|_| "media-bucket".to_string());
    let thumbnail_bucket_name =
        env::var("THUMBNAIL_BUCKET_NAME").unwrap_or_else(|_| "thumbnail-bucket".to_string());
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/media_db".to_string());
    let polling_schedule =
        env::var("POLLING_SCHEDULE").unwrap_or_else(|_| "0/5 * * * * *".to_string()); // Default to every 5 seconds

    // Initialize database connection
    let pool = PgPool::connect(&database_url).await?;
    let database = Database::new(pool);

    // Initialize thumbnail generator
    let thumbnail_generator =
        ThumbnailGenerator::new(s3_client.clone(), thumbnail_bucket_name.clone());

    // Initialize S3 poller
    let s3_poller = S3Poller::new(
        s3_client,
        bucket_name,
        thumbnail_bucket_name,
        database,
        thumbnail_generator,
    );

    // Start the polling scheduler
    s3_poller.start_polling(&polling_schedule).await?;

    info!("Media ingestion service started successfully");

    // Keep the service running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down media ingestion service");

    Ok(())
}
