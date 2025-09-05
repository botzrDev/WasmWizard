// src/middleware/redis_rate_limit.rs
use crate::middleware::rate_limit::RateLimit;
use crate::services::RedisService;
use anyhow::Result;
use uuid::Uuid;

/// Redis-based rate limiter
#[derive(Clone)]
pub struct RedisRateLimiter {
    redis: RedisService,
}

impl RedisRateLimiter {
    pub fn new(redis: RedisService) -> Self {
        Self { redis }
    }

    /// Get Redis key for a user's minute rate limit
    fn get_minute_key(&self, api_key_id: Uuid) -> String {
        format!("rate_limit:minute:{}", api_key_id)
    }

    /// Get Redis key for a user's daily rate limit
    fn get_day_key(&self, api_key_id: Uuid) -> String {
        format!("rate_limit:day:{}", api_key_id)
    }

    /// Check if a request is allowed, returns (allowed, retry_after_secs)
    pub async fn check_rate_limit(
        &self,
        api_key_id: Uuid,
        rate_limit: &RateLimit,
    ) -> Result<(bool, u64)> {
        let minute_key = self.get_minute_key(api_key_id);
        let day_key = self.get_day_key(api_key_id);

        // Increment and set expiry for minute counter
        let minute_count = self.redis.incr_and_expire(&minute_key, 60).await?;

        // Increment and set expiry for day counter
        let day_count = self.redis.incr_and_expire(&day_key, 86400).await?;

        // Check if either limit is exceeded
        let minute_exceeded = minute_count > rate_limit.requests_per_minute as i64;
        let day_exceeded = day_count > rate_limit.requests_per_day as i64;

        // Calculate retry-after if needed
        let retry_after = if minute_exceeded {
            // Get TTL for minute key
            let ttl = self.redis.ttl(&minute_key).await?;
            ttl as u64
        } else if day_exceeded {
            // Get TTL for day key
            let ttl = self.redis.ttl(&day_key).await?;
            ttl as u64
        } else {
            0
        };

        // Return whether the request is allowed and the retry-after time
        Ok((!minute_exceeded && !day_exceeded, retry_after))
    }

    /// Get remaining requests for an API key
    pub async fn get_remaining_requests(
        &self,
        api_key_id: Uuid,
        rate_limit: &RateLimit,
    ) -> Result<(u32, u32)> {
        let minute_key = self.get_minute_key(api_key_id);
        let day_key = self.get_day_key(api_key_id);

        // Get current counts or default to 0 if key doesn't exist
        let minute_count: i64 = match self.redis.get(&minute_key).await? {
            Some(val) => val.parse().unwrap_or(0),
            None => 0,
        };

        let day_count: i64 = match self.redis.get(&day_key).await? {
            Some(val) => val.parse().unwrap_or(0),
            None => 0,
        };

        // Calculate remaining
        let remaining_minute = rate_limit
            .requests_per_minute
            .saturating_sub(minute_count as u32);
        let remaining_day = rate_limit.requests_per_day.saturating_sub(day_count as u32);

        Ok((remaining_minute, remaining_day))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_rate_limiter() {
        // Skip test if REDIS_URL env var is not set
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Redis rate limiter test, REDIS_URL not set");
                return;
            }
        };

        // Create Redis service and rate limiter
        let redis = match RedisService::new(&redis_url) {
            Ok(service) => service,
            Err(e) => {
                println!("Skipping Redis rate limiter test, could not connect: {}", e);
                return;
            }
        };

        // Ping Redis to make sure it's available
        let mut conn = match redis.get_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                println!("Skipping Redis rate limiter test, could not get connection: {}", e);
                return;
            }
        };

        // Only continue if we can connect to Redis
        let ping_result: Result<String, redis::RedisError> =
            redis::cmd("PING").query_async(&mut conn).await;
        if let Err(e) = ping_result {
            println!("Skipping Redis rate limiter test, server not responding: {}", e);
            return;
        }

        println!("Redis server available, running rate limiter tests");

        let rate_limiter = RedisRateLimiter::new(redis);

        // Create test rate limit and API key ID
        let rate_limit = RateLimit {
            requests_per_minute: 5,
            requests_per_day: 10,
        };
        let api_key_id = Uuid::new_v4();

        // Clean up any previous test data
        let minute_key = rate_limiter.get_minute_key(api_key_id);
        let day_key = rate_limiter.get_day_key(api_key_id);
        let mut conn = rate_limiter.redis.get_connection().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&minute_key)
            .arg(&day_key)
            .query_async(&mut conn)
            .await
            .unwrap();

        // Test initial state
        let (remaining_minute, remaining_day) = rate_limiter
            .get_remaining_requests(api_key_id, &rate_limit)
            .await
            .unwrap();
        assert_eq!(remaining_minute, 5);
        assert_eq!(remaining_day, 10);

        // Test minute limit
        for i in 0..5 {
            let (allowed, _) = rate_limiter
                .check_rate_limit(api_key_id, &rate_limit)
                .await
                .unwrap();
            assert!(allowed, "Request {} should be allowed", i + 1);
        }

        // The 6th request should be denied
        let (allowed, retry_after) = rate_limiter
            .check_rate_limit(api_key_id, &rate_limit)
            .await
            .unwrap();
        assert!(!allowed, "Request 6 should be denied");
        assert!(retry_after > 0, "Retry-after should be positive");

        // Check remaining requests
        let (remaining_minute, remaining_day) = rate_limiter
            .get_remaining_requests(api_key_id, &rate_limit)
            .await
            .unwrap();
        assert_eq!(remaining_minute, 0);
        assert_eq!(remaining_day, 4);
    }
}
