// Comprehensive rate limiting tests
use std::time::Duration;
use tokio::time::sleep;
use wasm-wizard::middleware::rate_limit::{RateLimit, TokenBucket};

#[test]
fn test_rate_limit_from_tier_name() {
    let free_tier = RateLimit::from_tier_name("Free");
    assert_eq!(free_tier.requests_per_minute, 10);
    assert_eq!(free_tier.requests_per_day, 500);

    let basic_tier = RateLimit::from_tier_name("Basic");
    assert_eq!(basic_tier.requests_per_minute, 100);
    assert_eq!(basic_tier.requests_per_day, 10000);

    let pro_tier = RateLimit::from_tier_name("Pro");
    assert_eq!(pro_tier.requests_per_minute, 500);
    assert_eq!(pro_tier.requests_per_day, 50000);

    // Unknown tier should default to Free
    let unknown_tier = RateLimit::from_tier_name("Unknown");
    assert_eq!(unknown_tier.requests_per_minute, 10);
    assert_eq!(unknown_tier.requests_per_day, 500);
}

#[test]
fn test_rate_limit_case_insensitive() {
    let free_tier = RateLimit::from_tier_name("free");
    assert_eq!(free_tier.requests_per_minute, 10);

    let basic_tier = RateLimit::from_tier_name("BASIC");
    assert_eq!(basic_tier.requests_per_minute, 100);

    let pro_tier = RateLimit::from_tier_name("pRo");
    assert_eq!(pro_tier.requests_per_minute, 500);
}

#[test]
fn test_token_bucket_creation() {
    let bucket = TokenBucket::new(10, Duration::from_secs(60));

    // Initial state should have full tokens
    assert_eq!(bucket.tokens, 10);
    assert_eq!(bucket.capacity, 10);
}

#[test]
fn test_token_bucket_consume_success() {
    let mut bucket = TokenBucket::new(5, Duration::from_secs(60));

    // Should be able to consume tokens when available
    assert!(bucket.try_consume());
    assert_eq!(bucket.tokens, 4);

    assert!(bucket.try_consume());
    assert_eq!(bucket.tokens, 3);
}

#[test]
fn test_token_bucket_consume_exhaustion() {
    let mut bucket = TokenBucket::new(2, Duration::from_secs(60));

    // Consume all tokens
    assert!(bucket.try_consume());
    assert!(bucket.try_consume());

    // Should fail when no tokens left
    assert!(!bucket.try_consume());
    assert_eq!(bucket.tokens, 0);

    // Still should fail
    assert!(!bucket.try_consume());
    assert_eq!(bucket.tokens, 0);
}

#[tokio::test]
async fn test_token_bucket_refill() {
    let mut bucket = TokenBucket::new(2, Duration::from_millis(100));

    // Consume all tokens
    assert!(bucket.try_consume());
    assert!(bucket.try_consume());
    assert!(!bucket.try_consume());

    // Wait for refill interval
    sleep(Duration::from_millis(150)).await;

    // Should have refilled one token
    assert!(bucket.try_consume());
    assert!(!bucket.try_consume()); // But not two

    // Wait for another refill
    sleep(Duration::from_millis(150)).await;
    assert!(bucket.try_consume());
}

#[tokio::test]
async fn test_token_bucket_partial_refill() {
    let mut bucket = TokenBucket::new(10, Duration::from_millis(100));

    // Consume 5 tokens
    for _ in 0..5 {
        assert!(bucket.try_consume());
    }
    assert_eq!(bucket.tokens, 5);

    // Wait for half a refill interval
    sleep(Duration::from_millis(50)).await;

    // Try to consume - should still work from remaining tokens
    assert!(bucket.try_consume());
    assert_eq!(bucket.tokens, 4);

    // Wait for full refill interval
    sleep(Duration::from_millis(100)).await;

    // Should have one more token
    assert_eq!(bucket.tokens, 5); // 4 + 1 from refill
}

#[tokio::test]
async fn test_token_bucket_max_capacity() {
    let mut bucket = TokenBucket::new(3, Duration::from_millis(50));

    // Don't consume any tokens, just wait for multiple refill periods
    sleep(Duration::from_millis(200)).await; // 4 refill periods

    // Should not exceed capacity
    assert_eq!(bucket.tokens, 3);

    // Should still be able to consume all tokens
    assert!(bucket.try_consume());
    assert!(bucket.try_consume());
    assert!(bucket.try_consume());
    assert!(!bucket.try_consume());
}

#[test]
fn test_token_bucket_zero_capacity() {
    let mut bucket = TokenBucket::new(0, Duration::from_secs(1));

    // Should never be able to consume from zero-capacity bucket
    assert!(!bucket.try_consume());
    assert_eq!(bucket.tokens, 0);
}

#[test]
fn test_token_bucket_large_capacity() {
    let mut bucket = TokenBucket::new(1000, Duration::from_secs(1));

    // Should be able to consume many tokens
    for _ in 0..1000 {
        assert!(bucket.try_consume());
    }

    // Should be exhausted
    assert!(!bucket.try_consume());
    assert_eq!(bucket.tokens, 0);
}

#[tokio::test]
async fn test_token_bucket_very_fast_refill() {
    let mut bucket = TokenBucket::new(1, Duration::from_millis(1));

    // Consume token
    assert!(bucket.try_consume());
    assert!(!bucket.try_consume());

    // Wait minimal time
    sleep(Duration::from_millis(5)).await;

    // Should have refilled
    assert!(bucket.try_consume());
}

#[test]
fn test_rate_limit_display() {
    let rate_limit = RateLimit {
        requests_per_minute: 100,
        requests_per_day: 10000,
    };

    let display_str = format!("{:?}", rate_limit);
    assert!(display_str.contains("100"));
    assert!(display_str.contains("10000"));
}

#[test]
fn test_rate_limit_clone() {
    let original = RateLimit {
        requests_per_minute: 50,
        requests_per_day: 5000,
    };

    let cloned = original.clone();
    assert_eq!(original.requests_per_minute, cloned.requests_per_minute);
    assert_eq!(original.requests_per_day, cloned.requests_per_day);
}

#[test]
fn test_token_bucket_edge_cases() {
    // Test with 1 token capacity
    let mut single_bucket = TokenBucket::new(1, Duration::from_secs(1));
    assert!(single_bucket.try_consume());
    assert!(!single_bucket.try_consume());

    // Test with very long refill interval
    let mut slow_bucket = TokenBucket::new(1, Duration::from_secs(3600)); // 1 hour
    assert!(slow_bucket.try_consume());
    assert!(!slow_bucket.try_consume());
}
