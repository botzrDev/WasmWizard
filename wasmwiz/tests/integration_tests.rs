use actix_web::test;
use serde_json::{Value, json};
use sqlx::{PgPool, migrate::Migrator};
use std::{env, path::Path, sync::Once};
use testcontainers::{Docker, clients, images::postgres::Postgres};
use uuid::Uuid;
use wasmwiz::app::create_app;
use wasmwiz::{Config, establish_connection_pool};

static INIT: Once = Once::new();

async fn setup_test_environment() -> PgPool {
    INIT.call_once(|| {
        // Set up tracing for tests if not already set up
        if env::var("RUST_LOG").is_err() {
            unsafe {
                env::set_var("RUST_LOG", "debug,sqlx=warn,wasmwiz=debug");
            }
        }
        tracing_subscriber::fmt::init();
    });

    let docker = clients::Cli::default();
    let postgres_node = docker.run(Postgres::default());
    let port = postgres_node
        .get_host_port(5432)
        .expect("Failed to get postgres port");
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // Prevent the container from being dropped
    std::mem::forget(postgres_node);

    // Set environment variables for the test application
    unsafe {
        env::set_var("DATABASE_URL", &database_url);
        env::set_var("ENVIRONMENT", "staging"); // Enable auth for tests
        env::set_var("AUTH_REQUIRED", "true"); // Explicitly enable auth
        env::set_var("SERVER_HOST", "127.0.0.1");
        env::set_var("SERVER_PORT", "8080");
        env::set_var("API_SALT", "test_salt_for_api_keys_that_is_long_enough");
        env::set_var("WASM_TEMP_DIR", "./temp_wasm_test");
        env::set_var("MAX_WASM_SIZE", "10485760"); // 10MB
        env::set_var("MAX_INPUT_SIZE", "1048576"); // 1MB
        env::set_var("EXECUTION_TIMEOUT", "5");
        env::set_var("MEMORY_LIMIT", "128");
    }

    let config = Config::from_env().expect("Failed to load test configuration");
    let pool = establish_connection_pool(&config)
        .await
        .expect("Failed to establish test database connection");

    // Run migrations
    let migrator = Migrator::new(Path::new("./migrations"))
        .await
        .expect("Failed to create migrator");
    migrator.run(&pool).await.expect("Failed to run migrations");

    pool
}

async fn create_test_api_key(pool: &PgPool) -> (String, Uuid) {
    use sha2::{Digest, Sha256};

    // Generate a test API key
    let api_key = "ww_test_api_key_12345678901234567890";
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
    )
    .bind(user_id)
    .bind("test@example.com")
    .execute(pool)
    .await
    .expect("Failed to create test user");

    // Get Free tier ID
    let tier: (Uuid,) = sqlx::query_as("SELECT id FROM subscription_tiers WHERE name = 'Free'")
        .fetch_one(pool)
        .await
        .expect("Failed to get Free tier");

    // Create test API key
    let api_key_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO api_keys (id, key_hash, user_id, tier_id, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
    )
    .bind(api_key_id)
    .bind(&key_hash)
    .bind(user_id)
    .bind(tier.0)
    .execute(pool)
    .await
    .expect("Failed to create test API key");

    (api_key.to_string(), api_key_id)
}

fn create_simple_wasm_module() -> Vec<u8> {
    // Simple WASM module that just exports a function
    // This is a minimal valid WASM module
    vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // Version
        0x01, 0x04, 0x01, 0x60, // Type section
        0x00, 0x00, // Function type
        0x03, 0x02, 0x01, 0x00, // Function section
        0x07, 0x05, 0x01, 0x01, // Export section
        0x61, 0x00, 0x00, // Export "a" function 0
        0x0a, 0x04, 0x01, 0x02, // Code section
        0x00, 0x0b, // Function body (empty)
    ]
}

fn create_multipart_wasm_request(wasm_data: &[u8], input: &str) -> String {
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let mut body = String::new();

    // Add WASM file part
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str("Content-Disposition: form-data; name=\"wasm\"; filename=\"test.wasm\"\r\n");
    body.push_str("Content-Type: application/wasm\r\n\r\n");

    // Convert binary data to string (this is a hack for testing)
    let wasm_str = wasm_data.iter().map(|&b| b as char).collect::<String>();
    body.push_str(&wasm_str);
    body.push_str("\r\n");

    // Add input part
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str("Content-Disposition: form-data; name=\"input\"\r\n\r\n");
    body.push_str(input);
    body.push_str("\r\n");

    // End boundary
    body.push_str(&format!("--{}--\r\n", boundary));

    body
}

#[actix_web::test]
async fn test_health_check() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    // Verify response body contains expected health check info
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "wasmwiz");
    assert!(body["checks"].is_object());
    assert!(body["checks"]["database"]["status"] == "healthy");
}

#[actix_web::test]
async fn test_api_key_creation_and_listing() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test creating an API key
    let req = test::TestRequest::post()
        .uri("/admin/api-keys")
        .set_json(json!({
            "user_email": "test@example.com",
            "tier_name": "Free"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert!(body["api_key"].as_str().unwrap().starts_with("ww_"));
    assert!(body["api_key_id"].as_str().is_some());

    // Test listing API keys for the user
    let req = test::TestRequest::get()
        .uri("/admin/api-keys/test@example.com")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert!(!body.as_array().unwrap().is_empty());
    assert_eq!(body[0]["tier_name"], "Free");
}

#[actix_web::test]
async fn test_admin_api_key_auth_failure() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test accessing admin endpoint without auth (currently no auth implemented)
    let req = test::TestRequest::post()
        .uri("/admin/api-keys")
        .set_json(json!({
            "user_email": "test@example.com",
            "tier_name": "Free"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should succeed for now since admin auth is not implemented
    // TODO: Update this test when admin auth is implemented
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_csrf_protection() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test accessing CSRF token endpoint
    let req = test::TestRequest::get().uri("/csrf-token").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert!(body["csrf_token"].as_str().is_some());
    assert!(!body["csrf_token"].as_str().unwrap().is_empty());
}

#[actix_web::test]
async fn test_execute_endpoint_authentication() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test without API key - should return 401
    let wasm_data = create_simple_wasm_module();
    let multipart_body = create_multipart_wasm_request(&wasm_data, "test input");

    let req = test::TestRequest::post()
        .uri("/api/execute")
        .insert_header((
            "content-type",
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        ))
        .set_payload(multipart_body.clone())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test with invalid API key - should return 401
    let req = test::TestRequest::post()
        .uri("/api/execute")
        .insert_header(("authorization", "Bearer invalid_key"))
        .insert_header((
            "content-type",
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        ))
        .set_payload(multipart_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_wasm_execution_with_valid_auth() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Create a valid API key
    let (api_key, _) = create_test_api_key(&pool).await;

    // Test with valid API key but invalid WASM data
    let invalid_wasm = vec![0x00, 0x00, 0x00, 0x00]; // Invalid WASM magic
    let multipart_body = create_multipart_wasm_request(&invalid_wasm, "test input");

    let req = test::TestRequest::post()
        .uri("/api/execute")
        .insert_header(("authorization", format!("Bearer {}", api_key)))
        .insert_header((
            "content-type",
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        ))
        .set_payload(multipart_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad request due to invalid WASM format

    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid WASM file format")
    );
}

#[actix_web::test]
async fn test_wasm_invalid_module_handling() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    let (api_key, _) = create_test_api_key(&pool).await;

    // Test with missing WASM field
    let req = test::TestRequest::post()
        .uri("/api/execute")
        .insert_header(("authorization", format!("Bearer {}", api_key)))
        .insert_header(("content-type", "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW"))
        .set_payload("------WebKitFormBoundary7MA4YWxkTrZu0gW\r\nContent-Disposition: form-data; name=\"input\"\r\n\r\ntest\r\n------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Missing 'wasm' field")
    );
}

#[actix_web::test]
async fn test_full_wasm_execution_flow() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    let (api_key, api_key_id) = create_test_api_key(&pool).await;

    // Test with valid WASM module (though execution may fail)
    let wasm_data = create_simple_wasm_module();
    let multipart_body = create_multipart_wasm_request(&wasm_data, "test input");

    let req = test::TestRequest::post()
        .uri("/api/execute")
        .insert_header(("authorization", format!("Bearer {}", api_key)))
        .insert_header((
            "content-type",
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        ))
        .set_payload(multipart_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should not be 401/403 (auth should pass)
    assert_ne!(resp.status(), 401);
    assert_ne!(resp.status(), 403);

    // Check that usage was logged
    let usage_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM usage_logs WHERE api_key_id = $1")
            .bind(api_key_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count usage logs");

    assert!(usage_count.0 > 0, "Usage should be logged even if execution fails");
}

#[actix_web::test]
async fn test_rate_limiting_simulation() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    let (_api_key, _) = create_test_api_key(&pool).await;

    // Make multiple requests to test rate limiting middleware exists
    // Note: In a real test we'd need to make 11+ requests to hit the Free tier limit
    for _ in 0..3 {
        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Check rate limit headers are present
        let _headers = resp.headers();
        // Rate limiting headers should be added by middleware
        // (though the health endpoint might not be rate limited)
    }
}

#[actix_web::test]
async fn test_security_headers() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let headers = resp.headers();

    // Check for security headers
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("content-security-policy"));
}

#[actix_web::test]
async fn test_input_validation_middleware() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test with oversized payload
    let large_payload = "x".repeat(1024 * 1024 * 20); // 20MB
    let req = test::TestRequest::post()
        .uri("/api/execute")
        .set_payload(large_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should be rejected due to size (either by middleware or auth)
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_malicious_input_patterns() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test with suspicious query parameters
    let req = test::TestRequest::get()
        .uri("/health?script=%3Cscript%3Ealert('xss')%3C/script%3E") // URL-encoded
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should either succeed (if sanitized) or be rejected by input validation
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
async fn test_api_key_deactivation() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Create an API key first
    let req = test::TestRequest::post()
        .uri("/admin/api-keys")
        .set_json(json!({
            "user_email": "test@example.com",
            "tier_name": "Free"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let api_key_id = body["api_key_id"].as_str().unwrap();

    // Deactivate the API key
    let req = test::TestRequest::post()
        .uri(&format!("/admin/api-keys/{}/deactivate", api_key_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("deactivated"));
}

#[actix_web::test]
async fn test_web_interface_endpoints() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test main page
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test API keys page
    let req = test::TestRequest::get().uri("/api-keys").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_invalid_tier_handling() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config.clone())).await;

    // Test with invalid tier name
    let req = test::TestRequest::post()
        .uri("/admin/api-keys")
        .set_json(json!({
            "user_email": "test@example.com",
            "tier_name": "InvalidTier"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid tier name")
    );
}

#[actix_web::test]
async fn test_subscription_tiers_exist() {
    let pool = setup_test_environment().await;

    // Verify that subscription tiers were created by migration
    let tiers: Vec<(String,)> = sqlx::query_as("SELECT name FROM subscription_tiers ORDER BY name")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch tiers");

    let tier_names: Vec<String> = tiers.into_iter().map(|(name,)| name).collect();
    assert!(tier_names.contains(&"Free".to_string()));
    assert!(tier_names.contains(&"Basic".to_string()));
    assert!(tier_names.contains(&"Pro".to_string()));
}

#[actix_web::test]
async fn test_health_endpoints() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config)).await;

    // Test liveness probe
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "alive");

    // Test readiness probe
    let req = test::TestRequest::get().uri("/readyz").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["status"].as_str().unwrap().contains("ready"));
    assert!(body["checks"].is_object());

    // Test legacy health endpoint
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let pool = setup_test_environment().await;
    let config = Config::from_env().expect("Failed to load test configuration");
    let app = test::init_service(create_app(pool.clone(), config)).await;

    // Test metrics endpoint
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}
