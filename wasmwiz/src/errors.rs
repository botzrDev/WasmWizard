//! # Error Handling
//!
//! This module defines comprehensive error types for the Wasm Wizard application.
//! It provides structured error handling with appropriate HTTP status codes and
//! user-friendly error messages.
//!
//! ## Error Categories
//!
//! - **Client Errors (4xx)**: Bad requests, authentication failures, rate limiting
//! - **Server Errors (5xx)**: Internal errors, database failures, WASM execution errors
//! - **WASM-Specific Errors**: Compilation, instantiation, runtime, and resource limit errors
//!
//! ## HTTP Status Code Mapping
//!
//! | Error Type | HTTP Status | Description |
//! |------------|-------------|-------------|
//! | `BadRequest` | 400 | Invalid request data or parameters |
//! | `Unauthorized` | 401 | Missing or invalid authentication |
//! | `Forbidden` | 403 | Insufficient permissions |
//! | `NotFound` | 404 | Resource not found |
//! | `PayloadTooLarge` | 413 | Request payload exceeds limits |
//! | `TooManyRequests` | 429 | Rate limit exceeded |
//! | `UnprocessableEntity` | 422 | Valid request but cannot be processed |
//! | `InternalServerError` | 500 | Unexpected server error |
//!
//! ## Examples
//!
//! ```rust,no_run
//! use wasm-wizard::errors::ApiError;
//!
//! // Creating specific errors
//! let error = ApiError::BadRequest("Invalid WASM module format".to_string());
//!
//! // Converting from other error types
//! let db_error: Result<(), sqlx::Error> = Err(sqlx::Error::RowNotFound);
//! let api_error = ApiError::from(db_error.unwrap_err()); // Converts to ApiError::DbError
//! ```

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, From};
use sqlx::Error as SqlxError;
use std::error::Error as StdError;
use std::io::Error as IoError;
use wasmer::RuntimeError;
use wasmer_wasix::WasiError;

/// Comprehensive error type for the Wasm Wizard API.
///
/// This enum covers all possible error conditions in the application, from client
/// errors to server failures and WASM-specific issues. Each variant includes
/// appropriate HTTP status codes and user-friendly error messages.
#[allow(dead_code)]
#[derive(Debug, Display, From)]
pub enum ApiError {
    /// Catch-all for unexpected internal errors.
    /// Maps to HTTP 500 Internal Server Error.
    #[display(fmt = "Internal Server Error")]
    #[from]
    InternalError(anyhow::Error),

    /// Client provided invalid request data or parameters.
    /// Maps to HTTP 400 Bad Request.
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    /// Request lacks valid authentication credentials.
    /// Maps to HTTP 401 Unauthorized.
    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),

    /// Requested resource was not found.
    /// Maps to HTTP 404 Not Found.
    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),

    /// Authentication succeeded but user lacks required permissions.
    /// Maps to HTTP 403 Forbidden.
    #[display(fmt = "Forbidden: {}", _0)]
    Forbidden(String),

    /// Request is syntactically correct but cannot be processed.
    /// Maps to HTTP 422 Unprocessable Entity.
    #[display(fmt = "Unprocessable Entity: {}", _0)]
    UnprocessableEntity(String),

    /// Request payload exceeds configured size limits.
    /// Maps to HTTP 413 Payload Too Large.
    #[display(fmt = "Payload Too Large: {}", _0)]
    PayloadTooLarge(String),

    /// Client has exceeded rate limits.
    /// Maps to HTTP 429 Too Many Requests.
    #[display(fmt = "Rate Limit Exceeded: {}", _0)]
    TooManyRequests(String),

    /// Simplified rate limiting error without details.
    /// Maps to HTTP 429 Too Many Requests.
    #[display(fmt = "Rate Limited")]
    RateLimited,

    /// Failed to load or validate WASM module.
    /// Maps to HTTP 422 Unprocessable Entity.
    #[display(fmt = "Wasm Load Error: {}", _0)]
    WasmLoadError(String),

    /// WASM module executed but encountered a runtime error.
    /// Maps to HTTP 422 Unprocessable Entity.
    #[display(fmt = "Wasm Runtime Error: {}", _0)]
    WasmRuntimeError(String),

    /// WASM execution exceeded the configured time limit.
    /// Maps to HTTP 422 Unprocessable Entity.
    #[display(fmt = "Wasm Execution Time Limit Exceeded")]
    WasmTimeLimitExceeded,

    /// WASM execution exceeded the configured memory limit.
    /// Maps to HTTP 422 Unprocessable Entity.
    #[display(fmt = "Wasm Memory Limit Exceeded")]
    WasmMemoryLimitExceeded,

    /// Database operation failed.
    /// Maps to HTTP 500 Internal Server Error.
    #[display(fmt = "Database Error: {}", _0)]
    DbError(String),

    /// File system operation failed.
    /// Maps to HTTP 500 Internal Server Error.
    #[display(fmt = "File System Error: {}", _0)]
    FileIoError(String),
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ApiError::InternalError(err) => Some(err.root_cause()),
            _ => None,
        }
    }
}

impl ResponseError for ApiError {
    /// Converts the error into an HTTP response with appropriate status code and JSON body.
    ///
    /// # Returns
    ///
    /// Returns an `HttpResponse` with:
    /// - Appropriate HTTP status code based on error type
    /// - JSON body containing the error message
    /// - Rate limit headers for rate limiting errors
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_message = self.to_string(); // Uses the #[display] attribute

        let mut response = HttpResponse::build(status_code);

        // Add rate limit headers for rate limiting errors
        if let ApiError::TooManyRequests(_) = self {
            response
                .insert_header(("Retry-After", "60"))
                .insert_header((
                    "X-RateLimit-Reset",
                    (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + 60)
                        .to_string(),
                ));
        }

        response.json(serde_json::json!({
            "error": error_message,
        }))
    }

    /// Returns the appropriate HTTP status code for this error type.
    ///
    /// # Returns
    ///
    /// Returns the corresponding HTTP status code for the error variant.
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ApiError::UnprocessableEntity(_)
            | ApiError::WasmLoadError(_)
            | ApiError::WasmRuntimeError(_)
            | ApiError::WasmTimeLimitExceeded
            | ApiError::WasmMemoryLimitExceeded => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::InternalError(_) | ApiError::DbError(_) | ApiError::FileIoError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

// Implement From traits for easier error conversion
impl From<SqlxError> for ApiError {
    fn from(err: SqlxError) -> Self {
        ApiError::DbError(err.to_string())
    }
}

impl From<IoError> for ApiError {
    fn from(err: IoError) -> Self {
        ApiError::FileIoError(err.to_string())
    }
}

// Example conversion from Wasmer errors
impl From<wasmer::CompileError> for ApiError {
    fn from(err: wasmer::CompileError) -> Self {
        ApiError::WasmLoadError(format!("Failed to compile Wasm module: {}", err))
    }
}

impl From<wasmer::InstantiationError> for ApiError {
    fn from(err: wasmer::InstantiationError) -> Self {
        ApiError::WasmLoadError(format!("Failed to instantiate Wasm module: {}", err))
    }
}

impl From<RuntimeError> for ApiError {
    fn from(err: RuntimeError) -> Self {
        ApiError::WasmRuntimeError(format!("Wasm execution failed: {}", err))
    }
}

impl From<WasiError> for ApiError {
    fn from(err: WasiError) -> Self {
        ApiError::WasmRuntimeError(format!("WASI error during execution: {}", err))
    }
}

impl From<actix_multipart::MultipartError> for ApiError {
    fn from(err: actix_multipart::MultipartError) -> Self {
        ApiError::BadRequest(format!("Multipart parsing error: {}", err))
    }
}
