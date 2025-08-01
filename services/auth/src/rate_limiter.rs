//! Rate limiter for preventing brute force attacks

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::info;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of attempts allowed
    pub max_attempts: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Ban duration in seconds
    pub ban_duration_seconds: u64,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_seconds: 300,        // 5 minutes
            ban_duration_seconds: 3600, // 1 hour
        }
    }
}

/// Rate limiter entry
#[derive(Debug)]
struct RateLimiterEntry {
    /// Number of attempts
    attempts: u32,
    /// Last attempt time
    last_attempt: Instant,
    /// Ban expiration time
    ban_expires: Option<Instant>,
}

/// Rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Rate limiter configuration
    config: RateLimiterConfig,
    /// Rate limiter entries
    entries: Arc<Mutex<HashMap<String, RateLimiterEntry>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if an IP address is allowed to make a request
    pub async fn is_allowed(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();

        let entry = entries.entry(key.to_string()).or_insert(RateLimiterEntry {
            attempts: 0,
            last_attempt: now,
            ban_expires: None,
        });

        // Check if ban has expired
        if let Some(ban_expires) = entry.ban_expires {
            if now >= ban_expires {
                // Ban expired, reset attempts
                entry.attempts = 0;
                entry.ban_expires = None;
            } else {
                // Still banned
                return Ok(false);
            }
        }

        // Check if window has expired
        if now.duration_since(entry.last_attempt) >= Duration::from_secs(self.config.window_seconds)
        {
            // Window expired, reset attempts
            entry.attempts = 0;
        }

        // Check if we're over the limit
        if entry.attempts >= self.config.max_attempts {
            // Ban the IP
            entry.ban_expires = Some(now + Duration::from_secs(self.config.ban_duration_seconds));
            info!(
                "Banned key {} for {} seconds",
                key, self.config.ban_duration_seconds
            );
            return Ok(false);
        }

        // Increment attempts
        entry.attempts += 1;
        entry.last_attempt = now;

        Ok(true)
    }

    /// Get the rate limiter configuration
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }
}
