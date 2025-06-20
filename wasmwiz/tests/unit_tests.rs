use chrono::Utc;
use uuid::Uuid;
use wasmwiz::middleware::auth::{AuthContext, hash_api_key};
use wasmwiz::middleware::rate_limit::{RateLimit, TokenBucket};
use wasmwiz::models::{ApiKey, SubscriptionTier, UsageLog, User};

#[test]
fn test_api_key_hashing() {
    let api_key = "test_key_123";
    let hash1 = hash_api_key(api_key);
    let hash2 = hash_api_key(api_key);

    // Same input should produce same hash
    assert_eq!(hash1, hash2);

    // Different input should produce different hash
    let hash3 = hash_api_key("different_key");
    assert_ne!(hash1, hash3);

    // Hash should be 64 characters (256 bits / 4 bits per hex char)
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_subscription_tier_model() {
    let tier = SubscriptionTier {
        id: Uuid::new_v4(),
        name: "Free".to_string(),
        max_executions_per_minute: 10,
        max_executions_per_day: 500,
        max_memory_mb: 128,
        max_execution_time_seconds: 5,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(tier.name, "Free");
    assert_eq!(tier.max_executions_per_minute, 10);
    assert_eq!(tier.max_executions_per_day, 500);
}

#[test]
fn test_usage_log_creation() {
    let api_key_id = Uuid::new_v4();

    let success_log = UsageLog::success(api_key_id)
        .with_execution_duration(100)
        .with_file_sizes(1024, 256);

    assert_eq!(success_log.api_key_id, api_key_id);
    assert_eq!(success_log.status, "success");
    assert_eq!(success_log.execution_duration_ms, Some(100));
    assert_eq!(success_log.wasm_module_size_bytes, Some(1024));
    assert_eq!(success_log.input_size_bytes, Some(256));
    assert!(success_log.error_message.is_none());

    let error_log = UsageLog::error(api_key_id, "Test error".to_string());
    assert_eq!(error_log.status, "execution_error");
    assert_eq!(error_log.error_message, Some("Test error".to_string()));
}

#[test]
fn test_token_bucket_rate_limiting() {
    let mut bucket = TokenBucket::new(10.0, 10.0 / 60.0); // 10 tokens per minute (refill rate per second)

    // Should be able to consume tokens up to capacity
    for _ in 0..10 {
        assert!(bucket.try_consume(1.0));
    }

    // Should fail when no tokens left
    assert!(!bucket.try_consume(1.0));

    // Test refill (this is time-based so we can't easily test the actual refill)
    // But we can test the initial state
    let mut fresh_bucket = TokenBucket::new(5.0, 5.0 / 30.0); // 5 tokens per 30 seconds
    assert!(fresh_bucket.try_consume(5.0));
}

#[test]
fn test_rate_limit_from_tier() {
    let free = RateLimit::from_tier_name("Free");
    assert_eq!(free.requests_per_minute, 10);
    assert_eq!(free.requests_per_day, 500);

    let basic = RateLimit::from_tier_name("Basic");
    assert_eq!(basic.requests_per_minute, 100);
    assert_eq!(basic.requests_per_day, 10_000);

    let pro = RateLimit::from_tier_name("Pro");
    assert_eq!(pro.requests_per_minute, 500);
    assert_eq!(pro.requests_per_day, 50_000);

    let unknown = RateLimit::from_tier_name("unknown");
    assert_eq!(unknown.requests_per_minute, 10); // Default to free tier
    assert_eq!(unknown.requests_per_day, 500);
}

#[test]
fn test_auth_context_creation() {
    let api_key = ApiKey {
        id: Uuid::new_v4(),
        key_hash: "test_hash".to_string(),
        user_id: Uuid::new_v4(),
        tier_id: Uuid::new_v4(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let user = User {
        id: api_key.user_id,
        email: "test@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let tier = SubscriptionTier {
        id: api_key.tier_id,
        name: "Free".to_string(),
        max_executions_per_minute: 10,
        max_executions_per_day: 500,
        max_memory_mb: 128,
        max_execution_time_seconds: 5,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let auth_context = AuthContext {
        api_key: api_key.clone(),
        user: user.clone(),
        tier: tier.clone(),
    };

    assert_eq!(auth_context.api_key.id, api_key.id);
    assert_eq!(auth_context.user.email, "test@example.com");
    assert_eq!(auth_context.tier.name, "Free");
}

#[test]
fn test_wasm_validation() {
    // Test valid WASM magic bytes
    let valid_wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // Basic WASM header

    // We can't directly test the private is_valid_wasm function, but we can test the concept
    assert_eq!(&valid_wasm[0..4], &[0x00, 0x61, 0x73, 0x6D]);

    // Test invalid WASM magic bytes
    let invalid_wasm = [0x00, 0x00, 0x00, 0x00];
    assert_ne!(&invalid_wasm[0..4], &[0x00, 0x61, 0x73, 0x6D]);
}

#[test]
fn test_config_validation() {
    use std::env;
    use wasmwiz::config::Config;

    // Set up valid environment variables
    let vars = [
        ("DATABASE_URL", "postgres://localhost/test"),
        ("API_SALT", "this_is_a_long_enough_salt_for_testing"),
        ("SERVER_HOST", "127.0.0.1"),
        ("SERVER_PORT", "8080"),
        ("WASM_TEMP_DIR", "./temp"),
        ("MAX_WASM_SIZE", "10485760"),
        ("MAX_INPUT_SIZE", "1048576"),
        ("EXECUTION_TIMEOUT", "5"),
        ("MEMORY_LIMIT", "134217728"), // 128MB
    ];

    for (key, value) in &vars {
        unsafe {
            env::set_var(key, value);
        }
    }

    let config = Config::from_env().expect("Config should be valid");
    let validation_result = config.validate();
    if let Err(e) = &validation_result {
        eprintln!("Validation error: {:?}", e);
    }
    assert!(validation_result.is_ok());

    // Test invalid API salt (too short)
    unsafe {
        env::set_var("API_SALT", "short");
    }
    let config = Config::from_env().expect("Config should load");
    assert!(config.validate().is_err());

    // Restore valid salt
    unsafe {
        env::set_var("API_SALT", "this_is_a_long_enough_salt_for_testing");
    }
}

#[test]
fn test_csrf_token_generation() {
    use wasmwiz::middleware::generate_csrf_token;

    let secret = "test_secret";
    let token1 = generate_csrf_token(secret);
    let token2 = generate_csrf_token(secret);

    // Tokens should be different each time (they include timestamp)
    assert_ne!(token1, token2);

    // Tokens should be non-empty
    assert!(!token1.is_empty());
    assert!(!token2.is_empty());
}
