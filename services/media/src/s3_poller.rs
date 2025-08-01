use crate::database::Database;
use crate::metadata_extractor::MetadataExtractor;
use crate::models::{MediaItem, MediaMetadata, S3ObjectInfo};
use crate::thumbnail_generator::ThumbnailGenerator;
use anyhow::Result;
use aws_sdk_s3::{Client, types::Object};
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use std::time::Duration;
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

pub struct S3Poller {
    s3_client: Client,
    bucket_name: String,
    thumbnail_bucket_name: String,
    database: Database,
    thumbnail_generator: ThumbnailGenerator,
}

impl S3Poller {
    pub fn new(
        s3_client: Client,
        bucket_name: String,
        thumbnail_bucket_name: String,
        database: Database,
        thumbnail_generator: ThumbnailGenerator,
    ) -> Self {
        Self {
            s3_client,
            bucket_name,
            thumbnail_bucket_name,
            database,
            thumbnail_generator,
        }
    }

    pub async fn poll_bucket(&self) -> Result<Vec<S3ObjectInfo>> {
        info!("Polling S3 bucket: {}", self.bucket_name);

        let mut objects = Vec::new();
        let mut continuation_token = None;

        // Get list of already processed files from database
        let processed_files = self.database.get_processed_files().await?;
        let processed_files_set: std::collections::HashSet<String> =
            processed_files.into_iter().collect();

        loop {
            let mut request = self.s3_client.list_objects_v2().bucket(&self.bucket_name);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for obj in contents {
                    if let Some(key) = &obj.key {
                        // Skip if already processed
                        if processed_files_set.contains(key) {
                            continue;
                        }

                        // Convert to our S3ObjectInfo struct
                        let object_info = S3ObjectInfo {
                            key: key.clone(),
                            etag: obj.e_tag.clone().unwrap_or_default(),
                            last_modified: obj
                                .last_modified
                                .map(|dt| {
                                    chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                                        .unwrap_or_else(Utc::now)
                                })
                                .unwrap_or_else(Utc::now),
                            size: obj.size.unwrap_or(0) as u64,
                        };

                        objects.push(object_info);
                    }
                }
            }

            // Check if there are more objects to fetch
            if response.is_truncated.unwrap_or(false) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        info!("Found {} new objects in S3 bucket", objects.len());
        Ok(objects)
    }

    pub async fn process_objects(&self, objects: Vec<S3ObjectInfo>) -> Result<()> {
        info!("Processing {} objects", objects.len());

        for object in objects {
            // Implement retry mechanism with exponential backoff
            let max_retries = 3;
            let mut retry_count = 0;
            let mut success = false;

            while retry_count < max_retries && !success {
                match self.process_single_object(&object).await {
                    Ok(_) => {
                        success = true;
                        info!("Successfully processed object: {}", object.key);
                    }
                    Err(e) => {
                        error!(
                            "Failed to process object {} (attempt {}/{}): {}",
                            object.key,
                            retry_count + 1,
                            max_retries,
                            e
                        );
                        retry_count += 1;
                        if retry_count < max_retries {
                            // Exponential backoff: 1s, 2s, 4s
                            let delay = Duration::from_secs(2u64.pow(retry_count as u32));
                            sleep(delay).await;
                        }
                    }
                }
            }

            if !success {
                error!(
                    "Failed to process object {} after {} attempts",
                    object.key, max_retries
                );
                // TODO: Add to a dead letter queue or alerting system
            }
        }

        Ok(())
    }

    async fn process_single_object(&self, object: &S3ObjectInfo) -> Result<()> {
        // TODO: Download the file from S3 to a temporary location
        let temp_file_path = format!("/tmp/{}", object.key);

        // Extract metadata
        let metadata = MetadataExtractor::extract_metadata(&temp_file_path).await?;

        // Generate thumbnail
        let thumbnail_key = self
            .thumbnail_generator
            .generate_thumbnail(&object.key, &temp_file_path)
            .await?;

        // Create MediaItem with metadata and thumbnail URL
        let media_item = MediaItem {
            id: Uuid::new_v4(),
            media_type: "video".to_string(), // TODO: Determine media type from file extension or metadata
            metadata: serde_json::json!({
                "title": object.key.split('/').last().unwrap_or(&object.key), // Simple title from filename
                // Add other metadata fields as needed
            }),
            s3_key: object.key.clone(),
            status: "processed".to_string(),
            user_id: Uuid::new_v4(), // TODO: Associate with actual user
            created_at: Utc::now(),
            updated_at: Utc::now(),
            duration: metadata.duration,
            width: metadata.width,
            height: metadata.height,
            video_codec: metadata.video_codec,
            audio_codec: metadata.audio_codec,
            format: metadata.format,
            bitrate: metadata.bitrate,
            sample_rate: metadata.sample_rate,
            channels: metadata.channels,
            thumbnail_url: Some(format!(
                "s3://{}/{}",
                self.thumbnail_bucket_name, thumbnail_key
            )),
        };

        // Save to database
        self.database.save_media_item(&media_item).await?;

        // Mark object as processed
        self.database
            .mark_object_as_processed(&object.key, &object.etag)
            .await?;

        Ok(())
    }

    pub async fn start_polling(&self, schedule: &str) -> Result<()> {
        // Clone self for use in the async closure
        let poller = self.clone();

        let scheduler = JobScheduler::new().await?;

        let job = Job::new_async(schedule, move |_, _| {
            let poller = poller.clone();
            Box::pin(async move {
                info!("S3 polling job executed");
                match poller.poll_bucket().await {
                    Ok(objects) => {
                        if let Err(e) = poller.process_objects(objects).await {
                            error!("Failed to process objects: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to poll S3 bucket: {}", e);
                    }
                }
            })
        })?;

        scheduler.add(job).await?;
        scheduler.start().await?;

        info!("Started S3 polling scheduler with schedule: {}", schedule);
        Ok(())
    }
}

// Implement Clone for S3Poller
impl Clone for S3Poller {
    fn clone(&self) -> Self {
        Self {
            s3_client: self.s3_client.clone(),
            bucket_name: self.bucket_name.clone(),
            thumbnail_bucket_name: self.thumbnail_bucket_name.clone(),
            database: self.database.clone(),
            thumbnail_generator: self.thumbnail_generator.clone(),
        }
    }
}
