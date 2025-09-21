// Comprehensive input validation tests
use actix_web::http::StatusCode;
use actix_web::{
    http::{header, Method},
    test, web, App, HttpResponse,
};
use wasm_wizard::middleware::input_validation::InputValidationMiddleware;

#[actix_web::test]
async fn test_request_size_validation() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::post().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    // Test normal request - should pass
    let req = test::TestRequest::post()
        .uri("/test")
        .set_payload("normal request")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_suspicious_user_agent_blocked() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let suspicious_agents = vec![
        "sqlmap/1.0",
        "nikto-scanner",
        "nmap-scripts",
        "python-requests/2.25.1",
        "wget/1.20.3",
        "curl/7.68.0",
        "go-http-client/1.1",
    ];

    for user_agent in suspicious_agents {
        let req = test::TestRequest::get()
            .uri("/test")
            .append_header(("User-Agent", user_agent))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "Should block suspicious User-Agent: {}",
            user_agent
        );
    }
}

#[actix_web::test]
async fn test_legitimate_user_agent_allowed() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let legitimate_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Chrome/91.0.4472.124",
        "PostmanRuntime/7.28.0",
        "insomnia/2021.4.1",
        "HTTPie/2.4.0",
        "axios/0.21.1",
    ];

    for user_agent in legitimate_agents {
        let req = test::TestRequest::get()
            .uri("/test")
            .append_header(("User-Agent", user_agent))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Should allow legitimate User-Agent: {}",
            user_agent
        );
    }
}

#[actix_web::test]
async fn test_missing_user_agent_allowed() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_case_insensitive_user_agent_detection() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let case_variants = vec![
        "SQLMAP/1.0",
        "SqlMap",
        "sQLmAp",
        "PYTHON-REQUESTS",
        "Python-Requests",
        "CURL",
    ];

    for user_agent in case_variants {
        let req = test::TestRequest::get()
            .uri("/test")
            .append_header(("User-Agent", user_agent))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "Should block case-insensitive suspicious User-Agent: {}",
            user_agent
        );
    }
}

#[actix_web::test]
async fn test_partial_match_user_agent() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    // Test that partial matches within larger strings are caught
    let embedded_suspicious = vec![
        "MyApp/1.0 (powered by sqlmap)",
        "Custom-Tool-using-python-requests/1.0",
        "AutoBot-with-curl-backend",
    ];

    for user_agent in embedded_suspicious {
        let req = test::TestRequest::get()
            .uri("/test")
            .append_header(("User-Agent", user_agent))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "Should block User-Agent with embedded suspicious pattern: {}",
            user_agent
        );
    }
}

#[actix_web::test]
async fn test_common_methods_allowed() {
    let app = test::init_service(
        App::new().wrap(InputValidationMiddleware::new()).route(
            "/test",
            web::route()
                .method(Method::GET)
                .method(Method::POST)
                .method(Method::PUT)
                .method(Method::DELETE)
                .method(Method::PATCH)
                .to(|| async { HttpResponse::Ok().body("success") }),
        ),
    )
    .await;

    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
    ];

    for method in methods {
        let req = test::TestRequest::default()
            .method(method.clone())
            .uri("/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK, "Method {:?} should be allowed", method);
    }
}

#[actix_web::test]
async fn test_health_endpoint_bypassed() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/health", web::get().to(|| async { HttpResponse::Ok().body("healthy") }))
            .route("/health/live", web::get().to(|| async { HttpResponse::Ok().body("alive") }))
            .route("/health/ready", web::get().to(|| async { HttpResponse::Ok().body("ready") })),
    )
    .await;

    let health_endpoints = vec!["/health", "/health/live", "/health/ready"];

    for endpoint in health_endpoints {
        // Even with suspicious user agent, health endpoints should work
        let req = test::TestRequest::get()
            .uri(endpoint)
            .append_header(("User-Agent", "sqlmap/malicious"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Health endpoint {} should bypass validation",
            endpoint
        );
    }
}

#[actix_web::test]
async fn test_empty_user_agent_header() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .append_header(("User-Agent", ""))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_whitespace_only_user_agent() {
    let app = test::init_service(
        App::new()
            .wrap(InputValidationMiddleware::new())
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .append_header(("User-Agent", "   "))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
