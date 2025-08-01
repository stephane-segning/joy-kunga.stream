use crate::models::MediaMetadata;
use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub async fn extract_metadata(file_path: &str) -> Result<MediaMetadata> {
        info!("Extracting metadata from file: {}", file_path);

        // Run ffprobe to get metadata in JSON format
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(file_path)
            .output()?;

        if !output.status.success() {
            error!("ffprobe failed with status: {:?}", output.status);
            return Err(anyhow::anyhow!("ffprobe failed"));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let ffprobe_data: serde_json::Value = serde_json::from_str(&json_str)?;

        // Parse the JSON output to extract relevant metadata
        let metadata = Self::parse_ffprobe_output(&ffprobe_data)?;

        Ok(metadata)
    }

    fn parse_ffprobe_output(ffprobe_data: &serde_json::Value) -> Result<MediaMetadata> {
        let mut metadata = MediaMetadata {
            duration: None,
            width: None,
            height: None,
            video_codec: None,
            audio_codec: None,
            format: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
        };

        // Extract format information
        if let Some(format) = ffprobe_data.get("format") {
            if let Some(duration_str) = format.get("duration").and_then(|v| v.as_str()) {
                metadata.duration = duration_str.parse::<f64>().ok();
            }

            if let Some(format_name) = format.get("format_name").and_then(|v| v.as_str()) {
                metadata.format = Some(format_name.to_string());
            }

            if let Some(bit_rate_str) = format.get("bit_rate").and_then(|v| v.as_str()) {
                metadata.bitrate = bit_rate_str.parse::<i64>().ok();
            }
        }

        // Extract stream information
        if let Some(streams) = ffprobe_data.get("streams").and_then(|v| v.as_array()) {
            for stream in streams {
                if let Some(codec_type) = stream.get("codec_type").and_then(|v| v.as_str()) {
                    match codec_type {
                        "video" => {
                            if let Some(width) = stream.get("width").and_then(|v| v.as_i64()) {
                                metadata.width = Some(width as i32);
                            }

                            if let Some(height) = stream.get("height").and_then(|v| v.as_i64()) {
                                metadata.height = Some(height as i32);
                            }

                            if let Some(codec_name) =
                                stream.get("codec_name").and_then(|v| v.as_str())
                            {
                                metadata.video_codec = Some(codec_name.to_string());
                            }
                        }
                        "audio" => {
                            if let Some(codec_name) =
                                stream.get("codec_name").and_then(|v| v.as_str())
                            {
                                metadata.audio_codec = Some(codec_name.to_string());
                            }

                            if let Some(sample_rate_str) =
                                stream.get("sample_rate").and_then(|v| v.as_str())
                            {
                                metadata.sample_rate = sample_rate_str.parse::<i32>().ok();
                            }

                            if let Some(channels) = stream.get("channels").and_then(|v| v.as_i64())
                            {
                                metadata.channels = Some(channels as i32);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(metadata)
    }
}
