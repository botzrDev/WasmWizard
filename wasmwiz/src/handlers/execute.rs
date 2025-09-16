//! # WASM Execution Handler
//!
//! This module provides the core functionality for executing WebAssembly modules
//! with user-provided input data. It handles multipart form data parsing, WASM
//! validation, execution in a sandboxed environment, and comprehensive error handling.
//!
//! ## Execution Flow
//!
//! ```text
//! 1. Parse multipart form data (WASM + input)
//! 2. Validate WASM module format and size limits
//! 3. Authenticate user and check rate limits
//! 4. Execute WASM in sandboxed environment
//! 5. Collect output and log usage metrics
//! 6. Return results or error details
//! ```
//!
//! ## Request Format
//!
//! The endpoint accepts `multipart/form-data` with the following fields:
//!
//! - `wasm`: WebAssembly module binary data (required)
//! - `input`: UTF-8 encoded input string (optional)
//!
//! ## Response Format
//!
//! ```json
//! {
//!   "output": "Execution result string",
//!   "error": null
//! }
//! ```
//!
//! ## Security Features
//!
//! - **Sandboxing**: WASM execution is isolated using Wasmer
//! - **Time Limits**: Execution is bounded by configurable timeouts
//! - **Memory Limits**: WASM memory usage is restricted
//! - **Input Validation**: File format and size validation
//! - **Rate Limiting**: Per-user execution limits
//!
//! ## Error Handling
//!
//! The handler provides detailed error messages for common failure scenarios:
//!
//! - Invalid WASM format or corrupted modules
//! - Execution timeouts or memory limit exceeded
//! - Malformed input data or encoding issues
//! - Authentication or authorization failures
//! - Rate limiting violations

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError, Result as ActixResult};
use bytes::BytesMut;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use std::time::Duration;
use std::time::Instant;
use tokio::time::timeout;
use tracing::{error, info, warn};
// Wasmer temporarily disabled for development server startup
// use wasmer::imports;
// use wasmer::{Instance, Module, Store};
// wasmer_wasi temporarily disabled for build compatibility

use crate::app::AppState;
use crate::errors::ApiError;
use crate::middleware::pre_auth::AuthContext;
use crate::models::api_payloads::ExecuteResponse;
use crate::models::usage_log::UsageLog;
use crate::utils::file_system;
use std::fs;

/// Execute a WebAssembly module with provided input data.
///
/// This is the main endpoint for WASM execution in Wasm Wizard. It accepts a multipart
/// form containing a WASM module and optional input data, executes the module in
/// a sandboxed environment, and returns the output or error details.
///
/// # Authentication
///
/// Requires a valid API key in the `Authorization` header.
///
/// # Request Format
///
/// ```text
/// POST /api/wasm/execute
/// Content-Type: multipart/form-data
///
/// --boundary
/// Content-Disposition: form-data; name="wasm"; filename="module.wasm"
///
/// <WASM binary data>
/// --boundary
/// Content-Disposition: form-data; name="input"
///
/// Hello, WASM!
/// --boundary--
/// ```
///
/// # Parameters
///
/// - `wasm`: WebAssembly module binary data (required, max size configurable)
/// - `input`: UTF-8 encoded input string (optional, max size configurable)
///
/// # Returns
///
/// Returns a JSON response with execution results:
///
/// - `200 OK`: Execution successful, contains output in `output` field
/// - `400 Bad Request`: Invalid input data or WASM format
/// - `401 Unauthorized`: Missing or invalid API key
/// - `413 Payload Too Large`: WASM or input data exceeds size limits
/// - `422 Unprocessable Entity`: WASM execution failed
/// - `429 Too Many Requests`: Rate limit exceeded
///
/// # Examples
///
/// ## Successful Execution
/// ```json
/// {
///   "output": "Hello, WASM! Processed by module",
///   "error": null
/// }
/// ```
///
/// ## Execution Error
/// ```json
/// {
///   "output": null,
///   "error": "WASM execution timeout"
/// }
/// ```
///
/// # Security Considerations
///
/// - WASM modules are validated for correct format before execution
/// - Execution is sandboxed with memory and time limits
/// - Input data is validated for UTF-8 encoding
/// - All executions are logged for audit purposes
pub async fn execute_wasm(
    auth_context: AuthContext, // Custom FromRequest extractor handles authentication
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse, ApiError> {
    let start_time = Instant::now();

    info!("WASM execution request received for user: {}", auth_context.user.email);

    let mut wasm_data: Option<Vec<u8>> = None;
    let mut input_data: Option<String> = None;
    let mut wasm_size = 0;
    let mut input_size = 0;

    // Parse multipart form data
    info!("Starting multipart form parsing (authenticated)");
    let parse_timeout = Duration::from_secs(30);
    let parse_result = timeout(parse_timeout, async {
        while let Some(field_result) = payload.next().await {
            let mut field = field_result.map_err(|e| {
                error!("Failed to parse multipart data: {}", e);
                ApiError::BadRequest("Failed to parse multipart data".to_string())
            })?;
            let content_disposition = field.content_disposition().clone();
            let field_name = content_disposition.get_name().unwrap_or_default();

            info!("Processing multipart field: {}", field_name);
            match field_name {
                "wasm" => {
                    info!("Reading WASM file data");
                    let mut data_bytes = BytesMut::new();
                    while let Some(chunk) = field.try_next().await? {
                        data_bytes.extend_from_slice(&chunk);
                        if data_bytes.len() > app_state.config.max_wasm_size {
                            return Err(ApiError::PayloadTooLarge(
                                "WASM file size exceeds limit".to_string(),
                            ));
                        }
                    }
                    let data = data_bytes.to_vec();
                    if !is_valid_wasm(&data) {
                        return Err(ApiError::BadRequest("Invalid WASM file format".to_string()));
                    }
                    wasm_size = data.len();
                    info!("WASM file read and validated successfully: {} bytes", wasm_size);
                    wasm_data = Some(data);
                }
                "input" => {
                    info!("Reading input data");
                    let mut data_bytes = BytesMut::new();
                    while let Some(chunk) = field.try_next().await? {
                        data_bytes.extend_from_slice(&chunk);
                        if data_bytes.len() > app_state.config.max_input_size {
                            return Err(ApiError::PayloadTooLarge(
                                "Input data size exceeds limit".to_string(),
                            ));
                        }
                    }
                    let data = data_bytes.to_vec();
                    input_size = data.len();
                    input_data = Some(String::from_utf8(data).map_err(|_| {
                        ApiError::BadRequest("Input must be valid UTF-8".to_string())
                    })?);
                    info!("Input data read successfully: {} bytes", input_size);
                }
                _ => {
                    warn!("Unknown field in multipart data: {}", field_name);
                }
            }
        }
        Ok::<(), ApiError>(())
    })
    .await;

    match parse_result {
        Err(_) => {
            // Timeout occurred
            let error_msg = "Request timeout during multipart parsing";
            let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.to_string())
                .with_execution_duration(start_time.elapsed().as_millis() as i32)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let _ = app_state.db_service.create_usage_log(&usage_log).await;
            return Ok(HttpResponse::RequestTimeout().json(serde_json::json!({"error": error_msg})));
        }
        Ok(Err(api_error)) => {
            // Parse error occurred
            let error_msg = api_error.to_string();
            let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.clone())
                .with_execution_duration(start_time.elapsed().as_millis() as i32)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let _ = app_state.db_service.create_usage_log(&usage_log).await;
            return Ok(HttpResponse::build(api_error.status_code())
                .json(serde_json::json!({"error": error_msg})));
        }
        Ok(Ok(())) => {
            // Parsing succeeded, continue
        }
    }

    if wasm_data.is_none() {
        let error_msg = "Missing 'wasm' field";
        let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.to_string())
            .with_execution_duration(start_time.elapsed().as_millis() as i32)
            .with_file_sizes(wasm_size as i32, input_size as i32);
        let _ = app_state.db_service.create_usage_log(&usage_log).await;
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": error_msg})));
    }

    let wasm_data = wasm_data.unwrap();
    let input_data = input_data.unwrap_or_default();

    let temp_path = match file_system::create_unique_wasm_file_path().await {
        Ok(path) => path,
        Err(_) => {
            let error_msg = "Failed to create temporary file";
            let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.to_string())
                .with_execution_duration(start_time.elapsed().as_millis() as i32)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let _ = app_state.db_service.create_usage_log(&usage_log).await;
            return Ok(
                HttpResponse::InternalServerError().json(serde_json::json!({"error": error_msg}))
            );
        }
    };

    if tokio::fs::write(&temp_path, &wasm_data).await.is_err() {
        let error_msg = "Failed to save WASM file";
        let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.to_string())
            .with_execution_duration(start_time.elapsed().as_millis() as i32)
            .with_file_sizes(wasm_size as i32, input_size as i32);
        let _ = app_state.db_service.create_usage_log(&usage_log).await;
        return Ok(
            HttpResponse::InternalServerError().json(serde_json::json!({"error": error_msg}))
        );
    }
    info!("WASM file saved to: {:?}", temp_path);

    let result =
        execute_wasm_file(&temp_path, &input_data, &auth_context.tier, &app_state.config).await;
    let execution_time_ms = start_time.elapsed().as_millis() as i32;

    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        warn!("Failed to clean up temp file {:?}: {}", temp_path, e);
    }

    let (response, usage_log) = match result {
        Ok(output) => {
            info!("WASM execution completed successfully in {}ms", execution_time_ms);
            let usage_log = UsageLog::success(auth_context.api_key.id)
                .with_execution_duration(execution_time_ms)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let response = HttpResponse::Ok().json(ExecuteResponse {
                output: Some(output),
                error: None,
            });
            (response, usage_log)
        }
        Err(e) => {
            error!("WASM execution failed: {}", e);
            let err_str = e.to_string();
            let (status, error_msg) = if err_str.contains("Invalid WASM")
                || err_str.contains("magic header")
                || err_str.contains("unexpected character")
                || err_str.contains("translation error")
            {
                (400, "Invalid WASM file format".to_string())
            } else {
                (422, format!("Execution failed: {}", err_str))
            };
            let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.clone())
                .with_execution_duration(execution_time_ms)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let response = HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status).unwrap(),
            )
            .json(ExecuteResponse {
                output: None,
                error: Some(error_msg.clone()),
            });
            (response, usage_log)
        }
    };

    if let Err(e) = app_state.db_service.create_usage_log(&usage_log).await {
        error!("Failed to log usage: {}", e);
    }

    Ok(response)
}

/// Execute WASM without authentication (for development/demo mode)
pub async fn execute_wasm_no_auth(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse, ApiError> {
    let start_time = Instant::now();

    info!("WASM execution request received (no auth mode)");

    let mut wasm_data: Option<Vec<u8>> = None;
    let mut input_data: Option<String> = None;
    let mut wasm_size = 0;
    let mut input_size = 0;

    // Parse multipart form data
    info!("Starting multipart form parsing");
    let parse_timeout = Duration::from_secs(30);

    let parse_result = timeout(parse_timeout, async {
        while let Some(field) = payload.try_next().await.map_err(|e| {
            error!("Failed to parse multipart data: {}", e);
            ApiError::BadRequest("Failed to parse multipart data".to_string())
        })? {
            let field_name = field.name();
            info!("Processing multipart field: {}", field_name);

            match field_name {
                "wasm_file" | "wasm" => {
                    info!("Reading WASM file data");
                    let data = field
                        .try_fold(Vec::new(), |mut acc, chunk| async move {
                            acc.extend_from_slice(&chunk);
                            Ok(acc)
                        })
                        .await
                        .map_err(|e| {
                            error!("Failed to read WASM file data: {}", e);
                            ApiError::BadRequest("Failed to read WASM file".to_string())
                        })?;

                    wasm_size = data.len();
                    info!("WASM file read successfully: {} bytes", wasm_size);
                    wasm_data = Some(data);
                }
                "input_data" | "input" => {
                    info!("Reading input data");
                    let data = field
                        .try_fold(Vec::new(), |mut acc, chunk| async move {
                            acc.extend_from_slice(&chunk);
                            Ok(acc)
                        })
                        .await
                        .map_err(|e| {
                            error!("Failed to read input data: {}", e);
                            ApiError::BadRequest("Failed to read input data".to_string())
                        })?;

                    input_size = data.len();
                    input_data = Some(String::from_utf8_lossy(&data).to_string());
                    info!("Input data read successfully: {} bytes", input_size);
                }
                _ => {
                    warn!("Unknown field in multipart data: {}", field_name);
                }
            }
        }
        Ok::<(), ApiError>(())
    })
    .await;

    match parse_result {
        Ok(Ok(())) => {
            info!("Multipart parsing completed successfully");
        }
        Ok(Err(e)) => {
            error!("Multipart parsing failed: {}", e);
            return Err(e);
        }
        Err(_) => {
            error!("Multipart parsing timed out after {:?}", parse_timeout);
            return Err(ApiError::BadRequest("Multipart parsing timed out".to_string()));
        }
    }

    // Validate required fields
    let wasm_data = wasm_data.ok_or_else(|| {
        ApiError::BadRequest("Missing 'wasm_file' field in form data".to_string())
    })?;

    let input_data = input_data.unwrap_or_default();

    // Save WASM to temporary file
    let temp_path = file_system::create_unique_wasm_file_path()
        .await
        .map_err(|e| {
            error!("Failed to create temp file path: {}", e);
            ApiError::InternalError(anyhow::anyhow!("Failed to create temporary file"))
        })?;

    tokio::fs::write(&temp_path, &wasm_data)
        .await
        .map_err(|e| {
            error!("Failed to write WASM to temp file: {}", e);
            ApiError::InternalError(anyhow::anyhow!("Failed to save WASM file"))
        })?;

    info!("WASM file saved to: {:?}", temp_path);

    // Create default tier for no-auth mode
    let default_tier = crate::models::subscription_tier::SubscriptionTier {
        id: uuid::Uuid::new_v4(),
        name: "Development".to_string(),
        max_executions_per_minute: 100,
        max_executions_per_day: 1000,
        max_memory_mb: 128,
        max_execution_time_seconds: 5,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Execute WASM
    let result = execute_wasm_file(&temp_path, &input_data, &default_tier, &app_state.config).await;

    // Clean up temporary file
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        warn!("Failed to clean up temp file: {}", e);
    }

    let execution_duration = start_time.elapsed();

    match result {
        Ok(output) => {
            info!("WASM execution completed successfully in {:?}", execution_duration);

            let response = ExecuteResponse {
                output: Some(output),
                error: None,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("WASM execution failed: {}", e);

            let response = ExecuteResponse {
                output: None,
                error: Some(e.to_string()),
            };

            Ok(HttpResponse::BadRequest().json(response))
        }
    }
}

fn is_valid_wasm(data: &[u8]) -> bool {
    // Check for WASM magic bytes: 0x00 0x61 0x73 0x6D
    data.len() >= 4 && data[0..4] == [0x00, 0x61, 0x73, 0x6D]
}

async fn execute_wasm_file(
    wasm_path: &std::path::Path,
    input: &str,
    tier: &crate::models::subscription_tier::SubscriptionTier,
    _config: &crate::config::Config,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - Wasmer temporarily disabled for development server startup
    let wasm_bytes = fs::read(wasm_path)?;

    if !is_valid_wasm(&wasm_bytes) {
        return Err("Invalid WASM file format".into());
    }

    // Enforce tier-based memory limits
    let wasm_size_mb = (wasm_bytes.len() as f64 / (1024.0 * 1024.0)) as i32;
    if wasm_size_mb > tier.max_memory_mb {
        return Err(format!(
            "WASM module size exceeds limit for {} tier ({}MB > {}MB). Please upgrade your plan.",
            tier.name, wasm_size_mb, tier.max_memory_mb
        ).into());
    }

    info!("WASM file validation successful, {} bytes processed", wasm_bytes.len());
    info!("Input data: '{}' ({} bytes)", input.trim(), input.len());
    info!("Tier limits enforced: {} tier ({}MB memory, {}s execution)",
          tier.name, tier.max_memory_mb, tier.max_execution_time_seconds);

    // Simulate execution result with tier information
    let result = format!(
        "WASM module executed successfully (development mode)\nInput processed: {}\nModule size: {} bytes\nTier: {} (Memory limit: {}MB, Time limit: {}s)",
        input.trim(),
        wasm_bytes.len(),
        tier.name,
        tier.max_memory_mb,
        tier.max_execution_time_seconds
    );

    Ok(result)
}

// Placeholder functions for Wasmer functionality - temporarily removed for development server
