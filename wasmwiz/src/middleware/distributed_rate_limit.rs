// src/middleware/distributed_rate_limit.rs
// Distributed rate limiting using Redis

use actix_web::{HttpMessage, HttpRequest, Result as ActixResult};
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tracing::{debug, error, warn};
use crate::errors::ApiError;
use crate::middleware::auth::AuthContext;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_rate_limit(&self, key: &str, limit: u32, window: Duration) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

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
    async fn check_rate_limit(&self, key: &str, limit: u32, window: Duration) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        
        let window_secs = window.as_secs();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Sliding window key
        let window_key = format!("{}:{}", key, current_time / window_secs);
        
        // Use Redis INCR and EXPIRE for atomic operation
        let count: u32 = conn.incr(&window_key, 1).await?;
        
        if count == 1 {
            // Set expiration only for new keys
            let _: () = conn.expire(&window_key, window_secs as i64).await?;
        }
        
        debug!("Rate limit check for {}: {}/{}", key, count, limit);
        Ok(count <= limit)
    }
}

// Fallback in-memory rate limiter
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MemoryRateLimiter {
    counters: Arc<Mutex<HashMap<String, (u32, std::time::Instant)>>>,
}

impl MemoryRateLimiter {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RateLimiter for MemoryRateLimiter {
    async fn check_rate_limit(&self, key: &str, limit: u32, window: Duration) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
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
    limiter: Box<dyn RateLimiter>,
}

impl RateLimitService {
    pub fn new(limiter: Box<dyn RateLimiter>) -> Self {
        Self { limiter }
    }
    
    pub async fn check_request(&self, req: &HttpRequest) -> ActixResult<(), ApiError> {
        // Get user context for rate limiting
        let extensions = req.extensions();
        let auth_context = extensions.get::<AuthContext>();
        
        let rate_limit_key = if let Some(context) = auth_context {
            format!("user:{}", context.user.id)
        } else {
            // Use IP address for unauthenticated requests
            let connection_info = req.connection_info();
            format!("ip:{}", connection_info.peer_addr().unwrap_or("unknown"))
        };
        
        // Default rate limits (can be made configurable)
        let limit = if auth_context.is_some() { 60 } else { 10 }; // requests per minute
        let window = Duration::from_secs(60);
        
        match self.limiter.check_rate_limit(&rate_limit_key, limit, window).await {
            Ok(allowed) => {
                if allowed {
                    Ok(())
                } else {
                    warn!("Rate limit exceeded for key: {}", rate_limit_key);
                    Err(ApiError::RateLimited)
                }
            }
            Err(e) => {
                error!("Rate limiting error: {}", e);
                // Fail open - allow request if rate limiting fails
                warn!("Rate limiting service unavailable, allowing request");
                Ok(())
            }
        }
    }
}

// Factory function to create appropriate rate limiter based on configuration
pub fn create_rate_limiter(redis_url: Option<&str>) -> Box<dyn RateLimiter> {
    match redis_url {
        Some(url) => {
            match RedisRateLimiter::new(url) {
                Ok(redis_limiter) => {
                    debug!("Using Redis-based rate limiting");
                    Box::new(redis_limiter)
                }
                Err(e) => {
                    warn!("Failed to create Redis rate limiter: {}, falling back to memory", e);
                    Box::new(MemoryRateLimiter::new())
                }
            }
        }
        None => {
            debug!("Using memory-based rate limiting");
            Box::new(MemoryRateLimiter::new())
        }
    }
}
