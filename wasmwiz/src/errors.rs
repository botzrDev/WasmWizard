// src/errors.rs
// Defines custom error types for the application.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, From};
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use wasmer_wasi::WasiError;
use wasmer::RuntimeError;
use std::error::Error as StdError;
use anyhow;

#[allow(dead_code)]
#[derive(Debug, Display, From)]
pub enum ApiError {
    #[display(fmt = "Internal Server Error")]
    #[from]
    InternalError(anyhow::Error), // Catch-all for unexpected errors

    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden: {}", _0)]
    Forbidden(String),

    #[display(fmt = "Unprocessable Entity: {}", _0)]
    UnprocessableEntity(String),

    #[display(fmt = "Rate Limit Exceeded")]
    TooManyRequests,

    // Specific errors related to Wasm execution
    #[display(fmt = "Wasm Load Error: {}", _0)]
    WasmLoadError(String),

    #[display(fmt = "Wasm Runtime Error: {}", _0)]
    WasmRuntimeError(String),

    #[display(fmt = "Wasm Execution Time Limit Exceeded")]
    WasmTimeLimitExceeded,

    #[display(fmt = "Wasm Memory Limit Exceeded")]
    WasmMemoryLimitExceeded,

    // Database errors
    #[display(fmt = "Database Error: {}", _0)]
    DbError(String),

    // File I/O errors
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
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_message = self.to_string(); // Uses the #[display] attribute

        HttpResponse::build(status_code).json(serde_json::json!({
            "error": error_message,
        }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::UnprocessableEntity(_) |
            ApiError::WasmLoadError(_) |
            ApiError::WasmRuntimeError(_) |
            ApiError::WasmTimeLimitExceeded |
            ApiError::WasmMemoryLimitExceeded => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::InternalError(_) |
            ApiError::DbError(_) |
            ApiError::FileIoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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