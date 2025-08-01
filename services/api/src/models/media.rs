//! Media models for the API service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Media item model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub media_type: String,
    pub metadata: serde_json::Value,
    pub s3_key: String,
    pub status: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub duration: Option<f64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub format: Option<String>,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub thumbnail_url: Option<String>,
}

/// Query parameters for media listing
#[derive(Debug, Clone, Deserialize)]
pub struct MediaQuery {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort order (asc or desc)
    pub order: Option<String>,
    /// Filter by media type
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    /// Filter by status
    pub status: Option<String>,
    /// Filter by user ID
    pub user_id: Option<Uuid>,
    /// Search term for metadata
    pub search: Option<String>,
}

/// Response for media listing with pagination
#[derive(Debug, Clone, Serialize)]
pub struct MediaListResponse {
    pub items: Vec<MediaItem>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

/// Request for media refresh
#[derive(Debug, Clone, Deserialize)]
pub struct MediaRefreshRequest {
    /// Optional specific media item ID to refresh
    pub media_id: Option<Uuid>,
    /// Optional S3 key to refresh
    pub s3_key: Option<String>,
}
