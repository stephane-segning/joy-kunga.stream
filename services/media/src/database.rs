use crate::models::{MediaItem, S3ObjectInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, types::Uuid};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_processed_files(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT s3_key FROM processed_s3_objects")
            .fetch_all(&self.pool)
            .await?;

        let keys = rows.into_iter().map(|row| row.get("s3_key")).collect();

        Ok(keys)
    }

    pub async fn mark_object_as_processed(&self, s3_key: &str, etag: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO processed_s3_objects (s3_key, etag)
             VALUES ($1, $2)
             ON CONFLICT (s3_key) DO UPDATE SET
             etag = EXCLUDED.etag,
             processed_at = NOW(),
             updated_at = NOW()",
        )
        .bind(s3_key)
        .bind(etag)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_media_item(&self, item: &MediaItem) -> Result<()> {
        sqlx::query(
            "INSERT INTO media_items (id, type, metadata, s3_key, status, user_id, created_at, updated_at, duration, width, height, video_codec, audio_codec, format, bitrate, sample_rate, channels, thumbnail_url)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
             ON CONFLICT (id) DO UPDATE SET
             metadata = EXCLUDED.metadata,
             status = EXCLUDED.status,
             updated_at = EXCLUDED.updated_at,
             duration = EXCLUDED.duration,
             width = EXCLUDED.width,
             height = EXCLUDED.height,
             video_codec = EXCLUDED.video_codec,
             audio_codec = EXCLUDED.audio_codec,
             format = EXCLUDED.format,
             bitrate = EXCLUDED.bitrate,
             sample_rate = EXCLUDED.sample_rate,
             channels = EXCLUDED.channels,
             thumbnail_url = EXCLUDED.thumbnail_url"
        )
        .bind(&item.id)
        .bind(&item.media_type)
        .bind(&item.metadata)
        .bind(&item.s3_key)
        .bind(&item.status)
        .bind(&item.user_id)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .bind(&item.duration)
        .bind(&item.width)
        .bind(&item.height)
        .bind(&item.video_codec)
        .bind(&item.audio_codec)
        .bind(&item.format)
        .bind(&item.bitrate)
        .bind(&item.sample_rate)
        .bind(&item.channels)
        .bind(&item.thumbnail_url)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_media_item_status(&self, id: Uuid, status: &str) -> Result<()> {
        sqlx::query("UPDATE media_items SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
