// src/middleware/distributed_rate_limit.rs
// Distributed rate limiting using Redis

use crate::errors::ApiError;
use crate::middleware::pre_auth::AuthContext;
use actix_web::{HttpMessage, HttpRequest, Result as ActixResult};
use async_trait::async_trait;
use redis::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

/// Shared pointer type for rate limiters used by the middleware.
pub type SharedRateLimiter = Arc<dyn RateLimiter + Send + Sync + 'static>;

// Redis-based rate limiter
pub struct RedisRateLimiter {
    client: Client,
}

impl RedisRateLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let window_secs = window.as_secs();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Use a more sophisticated sliding window implementation
        // This uses a sorted set where score = timestamp
        let window_key = format!("rate:{}:{}", key, window_secs);

        // Execute a Lua script for atomic operations
        // This script:
        // 1. Adds current timestamp to sorted set
        // 2. Removes all entries outside the window
        // 3. Counts remaining entries
        // 4. Sets expiration time
        // 5. Returns whether under limit
        let script = redis::Script::new(
            r"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window_start = now - tonumber(ARGV[2])
            local limit = tonumber(ARGV[3])
            
            -- Add current timestamp
            redis.call('ZADD', key, now, now)
            
            -- Remove outdated entries
            redis.call('ZREMRANGEBYSCORE', key, 0, window_start)
            
            -- Count requests in window
            local count = redis.call('ZCARD', key)
            
            -- Set key expiration (2x window to be safe)
            redis.call('EXPIRE', key, tonumber(ARGV[2]) * 2)
            
            -- Return count and whether under limit
            return {count, count <= limit}
        ",
        );

        let result: (u32, bool) = script
            .key(window_key)
            .arg(current_time)
            .arg(window_secs)
            .arg(limit)
            .invoke_async(&mut conn)
            .await?;

        let (count, under_limit) = result;

        debug!("Rate limit check for {}: {}/{} (allowed: {})", key, count, limit, under_limit);
        Ok(under_limit)
    }
}

// Fallback in-memory rate limiter
use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct MemoryRateLimiter {
    counters: Arc<Mutex<HashMap<String, (u32, std::time::Instant)>>>,
}

impl MemoryRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemoryRateLimiter {
    fn default() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RateLimiter for MemoryRateLimiter {
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut counters = self.counters.lock().await;
        let now = std::time::Instant::now();

        // Clean up expired entries
        counters.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < window);

        let (count, _) = counters.entry(key.to_string()).or_insert((0, now));
        *count += 1;

        debug!("Rate limit check for {}: {}/{}", key, count, limit);
        Ok(*count <= limit)
    }
}

// Rate limiting middleware
pub struct RateLimitService {
    limiter: SharedRateLimiter,
}

impl Clone for RateLimitService {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
        }
    }
}

impl RateLimitService {
    pub fn new(limiter: SharedRateLimiter) -> Self {
        Self { limiter }
    }

    pub async fn check_request(&self, req: &HttpRequest) -> ActixResult<(), ApiError> {
        // Get user context for rate limiting
        let auth_context = {
            let extensions = req.extensions();
            extensions.get::<AuthContext>().cloned()
        };

        if let Some(context) = auth_context {
            // Authenticated request with API key
            let api_key_id = context.api_key.id.to_string();
            let _user_id = context.user.id.to_string();

            // Get tier-specific rate limits
            let minute_limit = context.tier.max_executions_per_minute as u32;
            let day_limit = context.tier.max_executions_per_day as u32;

            // Check per-minute limit
            let minute_key = format!("rate:{}:minute", api_key_id);
            let minute_window = Duration::from_secs(60);

            match self
                .limiter
                .check_rate_limit(&minute_key, minute_limit, minute_window)
                .await
            {
                Ok(allowed) => {
                    if !allowed {
                        warn!("Per-minute rate limit exceeded for API key: {}", api_key_id);
                        return Err(ApiError::TooManyRequests(format!(
                            "Rate limit exceeded: maximum {} requests per minute",
                            minute_limit
                        )));
                    }
                }
                Err(e) => {
                    error!("Rate limiting error: {}", e);
                    // Fail open - allow request if rate limiting fails
                    warn!("Rate limiting service unavailable, allowing request");
                }
            }

            // Check per-day limit
            let day_key = format!("rate:{}:day", api_key_id);
            let day_window = Duration::from_secs(86400); // 24 hours

            match self
                .limiter
                .check_rate_limit(&day_key, day_limit, day_window)
                .await
            {
                Ok(allowed) => {
                    if !allowed {
                        warn!("Per-day rate limit exceeded for API key: {}", api_key_id);
                        return Err(ApiError::TooManyRequests(format!(
                            "Rate limit exceeded: maximum {} requests per day",
                            day_limit
                        )));
                    }
                }
                Err(e) => {
                    error!("Rate limiting error: {}", e);
                    // Fail open - allow request if rate limiting fails
                    warn!("Rate limiting service unavailable, allowing request");
                }
            }

            debug!("Rate limits passed for API key: {}", api_key_id);
        } else {
            // Anonymous request - use IP address
            let ip = {
                let connection_info = req.connection_info();
                connection_info.peer_addr().unwrap_or("unknown").to_string()
            };
            let ip_key = format!("rate:ip:{}", ip);

            // Default anonymous limits
            let anon_minute_limit = 10; // 10 req/min
            let minute_window = Duration::from_secs(60);

            match self
                .limiter
                .check_rate_limit(&ip_key, anon_minute_limit, minute_window)
                .await
            {
                Ok(allowed) => {
                    if !allowed {
                        warn!("Anonymous rate limit exceeded for IP: {}", ip);
                        return Err(ApiError::TooManyRequests(
                            "Rate limit exceeded for anonymous requests".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Anonymous rate limiting error: {}", e);
                    // Fail open
                    warn!("Rate limiting service unavailable, allowing anonymous request");
                }
            }
        }

        Ok(())
    }
}

impl RateLimitService {
    #[cfg(test)]
    pub(crate) fn same_limiter_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.limiter, &other.limiter)
    }

    #[cfg(test)]
    pub(crate) fn shares_with(&self, other: &SharedRateLimiter) -> bool {
        Arc::ptr_eq(&self.limiter, other)
    }

    #[cfg(test)]
    pub(crate) async fn check_limit_for_test(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.limiter.check_rate_limit(key, limit, window).await
    }
}

// Factory function to create appropriate rate limiter based on configuration
pub fn create_rate_limiter(redis_url: Option<&str>) -> SharedRateLimiter {
    match redis_url {
        Some(url) => match RedisRateLimiter::new(url) {
            Ok(redis_limiter) => {
                debug!("Using Redis-based rate limiting");
                Arc::new(redis_limiter)
            }
            Err(e) => {
                warn!("Failed to create Redis rate limiter: {}, falling back to memory", e);
                Arc::new(MemoryRateLimiter::new())
            }
        },
        None => {
            debug!("Using memory-based rate limiting");
            Arc::new(MemoryRateLimiter::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_limiter_instance() {
        let limiter: SharedRateLimiter = Arc::new(MemoryRateLimiter::new());
        let service = RateLimitService::new(Arc::clone(&limiter));
        let cloned = service.clone();

        assert!(service.same_limiter_as(&cloned));
        assert!(service.shares_with(&limiter));
    }

    #[tokio::test]
    async fn clones_observe_shared_limits() {
        let limiter: SharedRateLimiter = Arc::new(MemoryRateLimiter::new());
        let service = RateLimitService::new(Arc::clone(&limiter));
        let cloned = service.clone();

        let key = "shared";
        let limit = 2;
        let window = Duration::from_secs(60);

        assert!(service
            .check_limit_for_test(key, limit, window)
            .await
            .unwrap());
        assert!(cloned
            .check_limit_for_test(key, limit, window)
            .await
            .unwrap());
        assert!(!service
            .check_limit_for_test(key, limit, window)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn concurrent_clones_share_limits() {
        let limiter: SharedRateLimiter = Arc::new(MemoryRateLimiter::new());
        let service = RateLimitService::new(Arc::clone(&limiter));
        let concurrent_clone = service.clone();

        let key = "concurrent";
        let limit = 3;
        let window = Duration::from_secs(60);

        let mut handles = Vec::new();
        for _ in 0..limit {
            let svc = service.clone();
            let key = key.to_string();
            handles.push(tokio::spawn(async move {
                svc.check_limit_for_test(&key, limit, window).await.unwrap()
            }));
        }

        let mut allowed = 0;
        for handle in handles {
            if handle.await.unwrap() {
                allowed += 1;
            }
        }

        assert_eq!(allowed, limit);
        assert!(!concurrent_clone
            .check_limit_for_test(key, limit, window)
            .await
            .unwrap());
    }
}
