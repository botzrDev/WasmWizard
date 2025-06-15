// src/middleware/rate_limit.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, Result, HttpMessage,
    http::header::{HeaderName, HeaderValue},
};
use futures_util::future::{ready, Ready, LocalBoxFuture};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use uuid::Uuid;
use crate::middleware::auth::AuthContext;

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
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
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
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
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            state: self.state.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    state: RateLimiterState,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let state = self.state.clone();

        Box::pin(async move {
            // Get authentication context
            let auth_context = match req.extensions().get::<AuthContext>() {
                Some(ctx) => ctx.clone(),
                None => {
                    tracing::warn!("Rate limit middleware called without authentication context");
                    let response = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal server error"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            let api_key_id = auth_context.api_key.id;
            let rate_limit = RateLimit::from_tier_name(&auth_context.tier.name);

            // Check rate limits
            let (allowed, retry_after) = {
                let mut state_guard = state.lock().unwrap();
                let buckets = state_guard.entry(api_key_id).or_insert_with(|| {
                    (
                        TokenBucket::new(rate_limit.requests_per_minute as f64, rate_limit.requests_per_minute as f64 / 60.0),
                        TokenBucket::new(rate_limit.requests_per_day as f64, rate_limit.requests_per_day as f64 / 86400.0),
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

                (allowed, retry_after)
            };

            if !allowed {
                let mut response = HttpResponse::TooManyRequests()
                    .json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "retry_after_seconds": retry_after
                    }));

                if retry_after > 0 {
                    response.headers_mut().insert(
                        HeaderName::from_static("retry-after"),
                        HeaderValue::from_str(&retry_after.to_string()).unwrap(),
                    );
                }

                return Ok(req.into_response(response));
            }

            // Add rate limit headers to successful requests
            let mut response = service.call(req).await?;
            
            // Add rate limit information to response headers
            let headers = response.headers_mut();
            headers.insert(
                HeaderName::from_static("x-ratelimit-limit-minute"),
                HeaderValue::from_str(&rate_limit.requests_per_minute.to_string()).unwrap(),
            );
            headers.insert(
                HeaderName::from_static("x-ratelimit-limit-day"),
                HeaderValue::from_str(&rate_limit.requests_per_day.to_string()).unwrap(),
            );

            // Get remaining requests from buckets
            let (remaining_minute, remaining_day) = {
                let state_guard = state.lock().unwrap();
                if let Some(buckets) = state_guard.get(&api_key_id) {
                    (buckets.0.tokens as u32, buckets.1.tokens as u32)
                } else {
                    (rate_limit.requests_per_minute, rate_limit.requests_per_day)
                }
            };

            headers.insert(
                HeaderName::from_static("x-ratelimit-remaining-minute"),
                HeaderValue::from_str(&remaining_minute.to_string()).unwrap(),
            );
            headers.insert(
                HeaderName::from_static("x-ratelimit-remaining-day"),
                HeaderValue::from_str(&remaining_day.to_string()).unwrap(),
            );

            Ok(response)
        })
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
