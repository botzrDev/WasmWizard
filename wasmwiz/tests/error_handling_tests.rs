// Comprehensive error handling tests
use actix_web::http::StatusCode;
use std::fmt::Write;
use wasm-wizard::errors::ApiError;

#[test]
fn test_api_error_bad_request() {
    let error = ApiError::BadRequest("Invalid input".to_string());

    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(error.to_string(), "Invalid input");
}

#[test]
fn test_api_error_unauthorized() {
    let error = ApiError::Unauthorized("Invalid API key".to_string());

    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(error.to_string(), "Invalid API key");
}

#[test]
fn test_api_error_forbidden() {
    let error = ApiError::Forbidden("Access denied".to_string());

    assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(error.to_string(), "Access denied");
}

#[test]
fn test_api_error_not_found() {
    let error = ApiError::NotFound("Resource not found".to_string());

    assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    assert_eq!(error.to_string(), "Resource not found");
}

#[test]
fn test_api_error_too_many_requests() {
    let error = ApiError::TooManyRequests("Rate limit exceeded".to_string());

    assert_eq!(error.status_code(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(error.to_string(), "Rate limit exceeded");
}

#[test]
fn test_api_error_payload_too_large() {
    let error = ApiError::PayloadTooLarge("File too big".to_string());

    assert_eq!(error.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(error.to_string(), "File too big");
}

#[test]
fn test_api_error_internal_error() {
    let internal_error = anyhow::anyhow!("Database connection failed");
    let error = ApiError::InternalError(internal_error);

    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(error.to_string().contains("Database connection failed"));
}

#[test]
fn test_api_error_validation_error() {
    let error = ApiError::ValidationError("Invalid WASM format".to_string());

    assert_eq!(error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error.to_string(), "Invalid WASM format");
}

#[test]
fn test_api_error_display() {
    let error = ApiError::BadRequest("Test error message".to_string());
    let displayed = format!("{}", error);
    assert_eq!(displayed, "Test error message");
}

#[test]
fn test_api_error_debug() {
    let error = ApiError::BadRequest("Test error".to_string());
    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("BadRequest"));
    assert!(debug_output.contains("Test error"));
}

#[test]
fn test_api_error_from_anyhow() {
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let api_error = ApiError::InternalError(anyhow_error);

    assert_eq!(api_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(api_error.to_string().contains("Something went wrong"));
}

#[test]
fn test_api_error_empty_messages() {
    let errors = vec![
        ApiError::BadRequest("".to_string()),
        ApiError::Unauthorized("".to_string()),
        ApiError::NotFound("".to_string()),
    ];

    for error in errors {
        assert_eq!(error.to_string(), "");
        assert!(error.status_code().as_u16() >= 400);
    }
}

#[test]
fn test_api_error_unicode_messages() {
    let unicode_message = "Error with unicode: 🚫❌💥";
    let error = ApiError::BadRequest(unicode_message.to_string());

    assert_eq!(error.to_string(), unicode_message);
    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_api_error_very_long_message() {
    let long_message = "a".repeat(10000);
    let error = ApiError::InternalError(anyhow::anyhow!("{}", long_message));

    assert!(error.to_string().len() > 1000);
    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_api_error_chain() {
    // Test nested error chain
    let root_cause = anyhow::anyhow!("Database connection timeout");
    let wrapped = root_cause.context("Failed to fetch user data");
    let api_error = ApiError::InternalError(wrapped);

    let error_string = api_error.to_string();
    assert!(error_string.contains("Failed to fetch user data"));
}

#[test]
fn test_api_error_status_codes_unique() {
    // Verify each error type has a unique status code
    let errors = vec![
        ApiError::BadRequest("test".to_string()),
        ApiError::Unauthorized("test".to_string()),
        ApiError::Forbidden("test".to_string()),
        ApiError::NotFound("test".to_string()),
        ApiError::TooManyRequests("test".to_string()),
        ApiError::PayloadTooLarge("test".to_string()),
        ApiError::ValidationError("test".to_string()),
        ApiError::InternalError(anyhow::anyhow!("test")),
    ];

    let mut status_codes = Vec::new();
    for error in errors {
        let status = error.status_code();
        assert!(!status_codes.contains(&status), "Duplicate status code found: {:?}", status);
        status_codes.push(status);
    }
}

#[test]
fn test_api_error_json_serializable() {
    // Test that error messages can be used in JSON responses
    let error = ApiError::BadRequest("Invalid JSON payload".to_string());
    let json_string = serde_json::json!({
        "error": error.to_string(),
        "status": error.status_code().as_u16()
    });

    assert!(json_string.is_object());
    assert_eq!(json_string["error"], "Invalid JSON payload");
    assert_eq!(json_string["status"], 400);
}

#[test]
fn test_api_error_special_characters() {
    let special_chars = r#"Special chars: "quotes", \backslashes\, and 🎉"#;
    let error = ApiError::BadRequest(special_chars.to_string());

    assert_eq!(error.to_string(), special_chars);

    // Test it can be serialized to JSON
    let json_result = serde_json::json!({
        "error": error.to_string()
    });
    assert!(json_result["error"].is_string());
}
