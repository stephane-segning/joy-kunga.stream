use crate::models::S3ObjectInfo;
use anyhow::Result;
use aws_sdk_s3::{Client, primitives::ByteStream};
use std::process::Command;
use tracing::{error, info};

#[derive(Clone)]
pub struct ThumbnailGenerator {
    s3_client: Client,
    thumbnail_bucket: String,
}

impl ThumbnailGenerator {
    pub fn new(s3_client: Client, thumbnail_bucket: String) -> Self {
        Self {
            s3_client,
            thumbnail_bucket,
        }
    }

    pub async fn generate_thumbnail(&self, video_key: &str, video_path: &str) -> Result<String> {
        info!("Generating thumbnail for video: {}", video_key);

        // Generate thumbnail filename from video key
        let thumbnail_key = Self::generate_thumbnail_key(video_key);

        // Generate thumbnail using FFmpeg
        let thumbnail_path = format!("/tmp/{}", thumbnail_key);
        self.extract_frame(video_path, &thumbnail_path).await?;

        // Upload thumbnail to S3
        self.upload_thumbnail(&thumbnail_path, &thumbnail_key)
            .await?;

        // Clean up temporary files
        std::fs::remove_file(thumbnail_path)?;

        Ok(thumbnail_key)
    }

    async fn extract_frame(&self, video_path: &str, thumbnail_path: &str) -> Result<()> {
        // Run FFmpeg to extract a frame at 10% of the video duration
        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(video_path)
            .arg("-ss")
            .arg("00:00:10") // Skip to 10 seconds (or 10% of video duration)
            .arg("-vframes")
            .arg("1")
            .arg("-f")
            .arg("image2")
            .arg(thumbnail_path)
            .output()?;

        if !output.status.success() {
            error!("FFmpeg failed with status: {:?}", output.status);
            return Err(anyhow::anyhow!("FFmpeg failed"));
        }

        Ok(())
    }

    async fn upload_thumbnail(&self, thumbnail_path: &str, thumbnail_key: &str) -> Result<()> {
        info!("Uploading thumbnail to S3: {}", thumbnail_key);

        let file_content = tokio::fs::read(thumbnail_path).await?;
        let byte_stream = ByteStream::from(file_content);

        self.s3_client
            .put_object()
            .bucket(&self.thumbnail_bucket)
            .key(thumbnail_key)
            .body(byte_stream)
            .content_type("image/jpeg")
            .send()
            .await?;

        Ok(())
    }

    fn generate_thumbnail_key(video_key: &str) -> String {
        // Replace the file extension with .jpg for the thumbnail
        let mut parts: Vec<&str> = video_key.split('.').collect();
        if parts.len() > 1 {
            parts.pop(); // Remove the original extension
        }
        parts.push("jpg"); // Add the thumbnail extension
        parts.join(".")
    }
}
