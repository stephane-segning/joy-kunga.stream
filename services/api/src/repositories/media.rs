//! Media repository for database operations

use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::media::{MediaItem, MediaQuery};

/// Media repository for database operations
#[derive(Clone)]
pub struct MediaRepository {
    pool: PgPool,
}

impl MediaRepository {
    /// Create a new media repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a media item by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT id, type, metadata, s3_key, status, user_id, created_at, updated_at,
                   duration, width, height, video_codec, audio_codec, format, bitrate,
                   sample_rate, channels, thumbnail_url
            FROM media_items
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let media_item = MediaItem {
                    id: row.get("id"),
                    media_type: row.get("type"),
                    metadata: row.get("metadata"),
                    s3_key: row.get("s3_key"),
                    status: row.get("status"),
                    user_id: row.get("user_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    duration: row.get("duration"),
                    width: row.get("width"),
                    height: row.get("height"),
                    video_codec: row.get("video_codec"),
                    audio_codec: row.get("audio_codec"),
                    format: row.get("format"),
                    bitrate: row.get("bitrate"),
                    sample_rate: row.get("sample_rate"),
                    channels: row.get("channels"),
                    thumbnail_url: row.get("thumbnail_url"),
                };
                Ok(Some(media_item))
            }
            None => Ok(None),
        }
    }

    /// Get media items with pagination, sorting, and filtering
    pub async fn get_media_items(&self, query: &MediaQuery) -> Result<(Vec<MediaItem>, i64)> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) as i64 * limit as i64;

        // Build a simple query for now - in a real implementation we would build dynamic queries
        let rows = sqlx::query(
            r#"
            SELECT id, type, metadata, s3_key, status, user_id, created_at, updated_at,
                   duration, width, height, video_codec, audio_codec, format, bitrate,
                   sample_rate, channels, thumbnail_url
            FROM media_items
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_items")
            .fetch_one(&self.pool)
            .await?;

        let media_items = rows
            .into_iter()
            .map(|row| MediaItem {
                id: row.get("id"),
                media_type: row.get("type"),
                metadata: row.get("metadata"),
                s3_key: row.get("s3_key"),
                status: row.get("status"),
                user_id: row.get("user_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                duration: row.get("duration"),
                width: row.get("width"),
                height: row.get("height"),
                video_codec: row.get("video_codec"),
                audio_codec: row.get("audio_codec"),
                format: row.get("format"),
                bitrate: row.get("bitrate"),
                sample_rate: row.get("sample_rate"),
                channels: row.get("channels"),
                thumbnail_url: row.get("thumbnail_url"),
            })
            .collect();

        Ok((media_items, count))
    }
}
