// src/middleware/rate_limit.rs
use crate::middleware::pre_auth::AuthContext;
use crate::middleware::redis_rate_limit::RedisRateLimiter;
use crate::services::RedisService;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderMap, HeaderName, HeaderValue},
    Error, HttpMessage, HttpResponse, Result,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Instant,
};
use uuid::Uuid;

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let time_passed = now.duration_since(self.last_refill).as_secs_f64();

        self.tokens = (self.tokens + time_passed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    fn get_retry_after(&self) -> u64 {
        let tokens_needed = 1.0 - self.tokens;
        if tokens_needed <= 0.0 {
            0
        } else {
            (tokens_needed / self.refill_rate).ceil() as u64
        }
    }
}

/// Rate limiter state for all API keys
type RateLimiterState = Arc<Mutex<HashMap<Uuid, (TokenBucket, TokenBucket)>>>; // (per_minute, per_day)

/// Rate limiting configuration per tier
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_day: u32,
}

impl RateLimit {
    pub fn from_tier_name(tier_name: &str) -> Self {
        match tier_name.to_lowercase().as_str() {
            "free" => Self {
                requests_per_minute: 10,
                requests_per_day: 500,
            },
            "basic" => Self {
                requests_per_minute: 100,
                requests_per_day: 10_000,
            },
            "pro" => Self {
                requests_per_minute: 500,
                requests_per_day: 50_000,
            },
            _ => Self {
                requests_per_minute: 10, // Default to most restrictive
                requests_per_day: 500,
            },
        }
    }
}

/// Rate limiting middleware factory
pub struct RateLimitMiddleware {
    state: RateLimiterState,
    redis_limiter: Option<RedisRateLimiter>,
    use_redis: bool,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            redis_limiter: None,
            use_redis: false,
        }
    }

    /// Create a new rate limiter with Redis support
    pub fn with_redis(redis: RedisService) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            redis_limiter: Some(RedisRateLimiter::new(redis)),
            use_redis: true,
        }
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            state: self.state.clone(),
            redis_limiter: self.redis_limiter.clone(),
            use_redis: self.use_redis,
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    state: RateLimiterState,
    redis_limiter: Option<RedisRateLimiter>,
    use_redis: bool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let state = self.state.clone();
        let redis_limiter = self.redis_limiter.clone();
        let use_redis = self.use_redis;

        Box::pin(async move {
            // Extract AuthContext - if not present, let the request continue
            // so that PreAuth middleware can handle authentication
            let auth_context = match req.extensions().get::<AuthContext>().cloned() {
                Some(context) => context,
                None => {
                    tracing::debug!(
                        "Rate limit middleware: no auth context found, proceeding without rate limiting"
                    );
                    return service
                        .call(req)
                        .await
                        .map(ServiceResponse::map_into_right_body);
                }
            };

            let api_key_id = auth_context.api_key.id;
            let rate_limit = RateLimit::from_tier_name(&auth_context.tier.name);

            // Check rate limits - using Redis if enabled, otherwise fallback to in-memory
            let (allowed, retry_after, remaining_minute, remaining_day) =
                if let (true, Some(limiter)) = (use_redis, redis_limiter) {
                    match limiter.check_rate_limit(api_key_id, &rate_limit).await {
                        Ok((allowed, retry_after)) => {
                            // Get remaining counts
                            let (rem_min, rem_day) = match limiter
                                .get_remaining_requests(api_key_id, &rate_limit)
                                .await
                            {
                                Ok((min, day)) => (min, day),
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to get remaining requests from Redis: {}",
                                        e
                                    );
                                    (0, 0) // Default to 0 on error
                                }
                            };
                            (allowed, retry_after, rem_min, rem_day)
                        }
                        Err(e) => {
                            tracing::error!("Redis rate limiter error: {}", e);
                            // Fallback to in-memory on Redis error
                            let (allowed, retry_after, rem_min, rem_day) =
                                check_in_memory_rate_limit(&state, api_key_id, &rate_limit);
                            (allowed, retry_after, rem_min, rem_day)
                        }
                    }
                } else {
                    // Use in-memory rate limiting
                    let (allowed, retry_after, rem_min, rem_day) =
                        check_in_memory_rate_limit(&state, api_key_id, &rate_limit);
                    (allowed, retry_after, rem_min, rem_day)
                };

            if !allowed {
                let mut response = HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "retry_after_seconds": retry_after
                }));

                if retry_after > 0 {
                    if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
                        response
                            .headers_mut()
                            .insert(HeaderName::from_static("retry-after"), value);
                    }
                }

                return Ok(req.into_response(response).map_into_left_body());
            }

            // Add rate limit headers to successful requests
            let mut response = service.call(req).await?.map_into_right_body();

            // Add rate limit information to response headers
            let headers = response.headers_mut();
            insert_header(headers, "x-ratelimit-limit-minute", rate_limit.requests_per_minute);
            insert_header(headers, "x-ratelimit-limit-day", rate_limit.requests_per_day);
            insert_header(headers, "x-ratelimit-remaining-minute", remaining_minute);
            insert_header(headers, "x-ratelimit-remaining-day", remaining_day);

            Ok(response)
        })
    }
}

/// Helper function for in-memory rate limiting
fn check_in_memory_rate_limit(
    state: &RateLimiterState,
    api_key_id: Uuid,
    rate_limit: &RateLimit,
) -> (bool, u64, u32, u32) {
    let mut state_guard = match state.lock() {
        Ok(guard) => guard,
        Err(error) => {
            tracing::error!("Rate limiter state lock poisoned: {}", error);
            error.into_inner()
        }
    };
    let buckets = state_guard.entry(api_key_id).or_insert_with(|| {
        (
            TokenBucket::new(
                rate_limit.requests_per_minute as f64,
                rate_limit.requests_per_minute as f64 / 60.0,
            ),
            TokenBucket::new(
                rate_limit.requests_per_day as f64,
                rate_limit.requests_per_day as f64 / 86400.0,
            ),
        )
    });

    let minute_allowed = buckets.0.try_consume(1.0);
    let day_allowed = buckets.1.try_consume(1.0);

    let allowed = minute_allowed && day_allowed;
    let retry_after = if !minute_allowed {
        buckets.0.get_retry_after()
    } else if !day_allowed {
        buckets.1.get_retry_after()
    } else {
        0
    };

    // Calculate remaining
    let remaining_minute = buckets.0.tokens as u32;
    let remaining_day = buckets.1.tokens as u32;

    (allowed, retry_after, remaining_minute, remaining_day)
}

fn insert_header<T: ToString>(headers: &mut HeaderMap, name: &'static str, value: T) {
    match HeaderValue::from_str(&value.to_string()) {
        Ok(header_value) => {
            headers.insert(HeaderName::from_static(name), header_value);
        }
        Err(error) => {
            tracing::error!("Failed to insert rate limit header {}: {}", name, error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 tokens, 1 per second

        // Should start with full capacity
        assert!(bucket.try_consume(10.0));
        assert!(!bucket.try_consume(1.0)); // No tokens left

        // Test refill (we can't easily test time passage, so this is basic)
        bucket.tokens = 5.0; // Manually set for testing
        assert!(bucket.try_consume(5.0));
        assert!(!bucket.try_consume(1.0));
    }

    #[test]
    fn test_rate_limit_from_tier() {
        let free = RateLimit::from_tier_name("free");
        assert_eq!(free.requests_per_minute, 10);
        assert_eq!(free.requests_per_day, 500);

        let basic = RateLimit::from_tier_name("basic");
        assert_eq!(basic.requests_per_minute, 100);
        assert_eq!(basic.requests_per_day, 10_000);

        let pro = RateLimit::from_tier_name("pro");
        assert_eq!(pro.requests_per_minute, 500);
        assert_eq!(pro.requests_per_day, 50_000);

        let unknown = RateLimit::from_tier_name("unknown");
        assert_eq!(unknown.requests_per_minute, 10); // Default to free tier
        assert_eq!(unknown.requests_per_day, 500);
    }
}
