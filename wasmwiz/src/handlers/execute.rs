// src/handlers/execute.rs
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web, ResponseError};
use futures_util::TryStreamExt;
use serde::Deserialize;
use serde_urlencoded;
use bytes::BytesMut;
use futures_util::StreamExt;
use std::time::Instant;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use wasmer::{Instance, Module, Store};
use wasmer::imports;

use crate::app::AppState;
use crate::errors::ApiError;
use crate::middleware::pre_auth::AuthContext;
use crate::models::api_payloads::ExecuteResponse;
use crate::models::usage_log::UsageLog;
use crate::utils::file_system;
use std::fs;

/// Execute a WebAssembly module with provided input
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
                            return Err(ApiError::PayloadTooLarge("WASM file size exceeds limit".to_string()));
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
                            return Err(ApiError::PayloadTooLarge("Input data size exceeds limit".to_string()));
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
    }).await;

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
            return Ok(HttpResponse::build(api_error.status_code()).json(serde_json::json!({"error": error_msg})));
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
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": error_msg})));
        }
    };

    if tokio::fs::write(&temp_path, &wasm_data).await.is_err() {
        let error_msg = "Failed to save WASM file";
        let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.to_string())
            .with_execution_duration(start_time.elapsed().as_millis() as i32)
            .with_file_sizes(wasm_size as i32, input_size as i32);
        let _ = app_state.db_service.create_usage_log(&usage_log).await;
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": error_msg})));
    }
    info!("WASM file saved to: {:?}", temp_path);

    let result = execute_wasm_file(&temp_path, &input_data, &auth_context.tier, &app_state.config).await;
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
            let (status, error_msg) = if err_str.contains("Invalid WASM") || err_str.contains("magic header") || err_str.contains("unexpected character") || err_str.contains("translation error") {
                (400, "Invalid WASM file format".to_string())
            } else {
                (422, format!("Execution failed: {}", err_str))
            };
            let usage_log = UsageLog::error(auth_context.api_key.id, error_msg.clone())
                .with_execution_duration(execution_time_ms)
                .with_file_sizes(wasm_size as i32, input_size as i32);
            let response = HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap()).json(ExecuteResponse {
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
    }).await;
    
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
    let wasm_data = wasm_data
        .ok_or_else(|| ApiError::BadRequest("Missing 'wasm_file' field in form data".to_string()))?;

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

#[derive(Deserialize, Debug)]
pub struct DebugForm {
    pub wasm: Option<String>,
    pub input: Option<String>,
}

/// Debug endpoint to test multipart and urlencoded handling
pub async fn debug_execute(
    req: HttpRequest,
    mut payload: web::Payload,
) -> ActixResult<HttpResponse, ApiError> {
    let content_type = req.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("");
    let mut fields = Vec::new();
    let start_time = Instant::now();

    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::new(&req.headers(), payload);
        let parse_timeout = Duration::from_secs(10);
        let parse_result = timeout(parse_timeout, async {
            let mut found_field = false;
            while let Some(field) = multipart.try_next().await.map_err(|e| {
                error!("Failed to parse multipart data: {}", e);
                ApiError::BadRequest("Failed to parse multipart data".to_string())
            })? {
                found_field = true;
                let field_name = field.name().to_string();
                info!("DEBUG FIELD NAME: {}", field_name);
                let field_start = Instant::now();
                let field_size = field.try_fold(0, |acc, chunk| async move {
                    Ok(acc + chunk.len())
                }).await.unwrap_or(0);
                let field_duration = field_start.elapsed();
                fields.push(format!("{}: {} bytes ({}ms)", field_name, field_size, field_duration.as_millis()));
                info!("Received field: {} ({} bytes) in {:?}", field_name, field_size, field_duration);
            }
            if !found_field {
                warn!("No fields found in multipart upload");
            }
            Ok::<(), ApiError>(())
        }).await;
        let total_duration = start_time.elapsed();
        match parse_result {
            Ok(Ok(())) => {
                let response = serde_json::json!({
                    "status": "debug_success",
                    "fields": fields,
                    "parse_duration_ms": total_duration.as_millis()
                });
                return Ok(HttpResponse::Ok().json(response));
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                let response = serde_json::json!({
                    "status": "debug_timeout",
                    "fields": fields,
                    "parse_duration_ms": total_duration.as_millis()
                });
                return Ok(HttpResponse::RequestTimeout().json(response));
            }
        }
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        let mut body = BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk.map_err(|e| ApiError::BadRequest(format!("Payload error: {}", e)))?;
            body.extend_from_slice(&chunk);
        }
        let form: DebugForm = serde_urlencoded::from_bytes(&body).map_err(|e| {
            error!("Failed to parse urlencoded body: {}", e);
            ApiError::BadRequest("Failed to parse urlencoded body".to_string())
        })?;
        if let Some(wasm) = &form.wasm {
            fields.push(format!("wasm: {} bytes", wasm.len()));
        }
        if let Some(input) = &form.input {
            fields.push(format!("input: {} bytes", input.len()));
        }
        let total_duration = start_time.elapsed();
        let response = serde_json::json!({
            "status": "debug_success",
            "fields": fields,
            "parse_duration_ms": total_duration.as_millis()
        });
        return Ok(HttpResponse::Ok().json(response));
    } else {
        return Err(ApiError::BadRequest("Unsupported content type for debug endpoint".to_string()));
    }
}

async fn collect_field_data(
    mut field: actix_multipart::Field,
    max_size: usize,
) -> Result<Vec<u8>, ApiError> {
    let mut data = Vec::new();

    while let Some(chunk) = field.try_next().await.map_err(|e| {
        error!("Failed to read field data: {}", e);
        ApiError::BadRequest("Failed to read field data".to_string())
    })? {
        if data.len() + chunk.len() > max_size {
            return Err(ApiError::BadRequest(format!(
                "Field data exceeds maximum size of {} bytes",
                max_size
            )));
        }
        data.extend_from_slice(&chunk);
    }

    Ok(data)
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
    // Read the WASM file
    let wasm_bytes = fs::read(wasm_path)?;

    // Set up the Wasmer store
    let mut store = Store::default();
    let module = Module::new(&store, &wasm_bytes)?;

    debug!("WASM module imports: {:?}", module.imports().collect::<Vec<_>>());
    debug!("WASM module exports: {:?}", module.exports().collect::<Vec<_>>());

    // Check WASI imports to determine module type
    let imports = module.imports().collect::<Vec<_>>();
    let wasi_imports = imports.iter()
        .filter(|import| import.module().starts_with("wasi"))
        .collect::<Vec<_>>();

    debug!("WASI imports found: {:?}", wasi_imports);
    
    // Check if this is a non-WASI WASM file first
    if wasi_imports.is_empty() {
        debug!("Module appears to be a non-WASI WASM file, attempting direct execution");
        return execute_non_wasi_wasm(&mut store, &module, input, tier).await;
    }

    debug!("Detected WASI module, returning informative message for now");

    // For now, return a detailed message about WASI support instead of trying to execute
    // This prevents hanging and gives useful information
    let import_details: Vec<String> = wasi_imports.iter()
        .map(|import| format!("{}::{}", import.module(), import.name()))
        .collect();

    let export_names: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
    
    Ok(format!(
        "WASI module detected!\n\
        Required imports: {}\n\
        Module exports: {}\n\
        \n\
        WASI execution is currently being implemented. \
        This module appears to be a WASI program that would normally print to stdout.\n\
        \n\
        Input received: '{}'\n\
        Module size: {} bytes",
        import_details.join(", "),
        export_names.join(", "),
        input,
        wasm_bytes.len()
    ))
}

// Fallback function for non-WASI WASM modules
async fn execute_non_wasi_wasm(
    store: &mut Store,
    module: &Module,
    input: &str,
    tier: &crate::models::subscription_tier::SubscriptionTier,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use wasmer::imports;
    use tracing::debug;
    
    debug!("Attempting to execute non-WASI WASM module");
    
    // Create a more comprehensive import object for non-WASI modules
    let import_object = imports! {
        "env" => {
            // Common browser-like JS functions that some WASM modules expect
            "console_log" => wasmer::Function::new_typed(store, |msg: i32| {
                debug!("console_log called with: {}", msg);
            }),
        },
        // Empty JS namespace for JS-compiled WASM modules
        "js" => {},
        // Empty wasi namespace as fallback
        "wasi" => {}
    };
    
    // Try to create instance with enhanced error reporting
    let instance = match Instance::new(store, module, &import_object) {
        Ok(instance) => {
            debug!("Non-WASI instance created successfully");
            instance
        },
        Err(e) => {
            debug!("Failed to create non-WASI instance: {}", e);
            // Try a completely empty import object as a last resort
            match Instance::new(store, module, &imports! {}) {
                Ok(empty_instance) => {
                    debug!("Created instance with empty imports");
                    empty_instance
                },
                Err(empty_err) => {
                    return Err(format!("Failed to create WASM instance with both custom imports and empty imports: {} / {}", 
                        e, empty_err).into());
                }
            }
        }
    };
    
    // Get all exports to better understand the module
    let exports = instance.exports;
    debug!("Module has the following exports: {:?}", exports.iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>());
    
    // Try common function names in priority order
    let function_names = [
        "main", "run", "execute", "start", "_start", 
        "initialize", "_initialize", "default", "wasmMain", "runWasm"
    ];
    
    let exec_timeout = Duration::from_secs(tier.max_execution_time_seconds as u64);
    
    // Try each function and return on first success
    for func_name in &function_names {
        if let Ok(func) = exports.get_function(func_name) {
            debug!("Found and calling function: {}", func_name);
            
            // Create a simpler execution approach that doesn't use spawn_blocking
            // since that can cause issues with store ownership
            match timeout(exec_timeout, async {
                // Try calling with no parameters first
                match func.call(store, &[]) {
                    Ok(result) => {
                        // Convert result to string if possible
                        if result.is_empty() {
                            Ok("Function executed successfully (no return value)".to_string())
                        } else if let Some(val) = result[0].i32() {
                            Ok(format!("Function returned: {}", val))
                        } else {
                            Ok(format!("Function returned: {:?}", result))
                        }
                    },
                    Err(e) => {
                        // If zero-param call fails, try with input length as parameter
                        debug!("Function call with no params failed: {}, trying with input length", e);
                        match func.call(store, &[wasmer::Value::I32(input.len() as i32)]) {
                            Ok(result) => {
                                if result.is_empty() {
                                    Ok("Function executed successfully with input length parameter (no return value)".to_string())
                                } else {
                                    Ok(format!("Function returned with input length parameter: {:?}", result))
                                }
                            },
                            Err(e2) => {
                                Err(format!("Function execution failed with both no params and input length: {} / {}", 
                                    e, e2).into())
                            }
                        }
                    }
                }
            }).await {
                Ok(output) => return output,
                Err(_) => {
                    debug!("Function {} execution timed out", func_name);
                    return Err("Function execution timed out".into());
                },
            };
        }
    }
    
    // If we get here, we tried all functions and none worked
    // Try to extract any exported strings or memory that might contain output
    if let Ok(memory) = exports.get_memory("memory") {
        debug!("No function executed successfully, looking for output in memory");
        let view = memory.view(store);
        
        // Look for null-terminated string at common output locations
        let potential_offsets = [0, 1024, 4096, 8192];
        for offset in potential_offsets {
            let mut bytes = Vec::new();
            for i in 0..1024 {  // Read up to 1KB from each offset
                if offset + i >= view.data_size() {
                    break; // Don't read beyond memory bounds
                }
                let byte = view.read_u8(offset + i as u64).unwrap_or(0);
                if byte == 0 {  // Null terminator
                    break;
                }
                bytes.push(byte);
            }
            
            if !bytes.is_empty() {
                if let Ok(s) = String::from_utf8(bytes) {
                    if !s.trim().is_empty() {
                        debug!("Found potential output string at offset {}: {}", offset, s);
                        return Ok(s);
                    }
                }
            }
        }
    }
    
    Err("No suitable entry point found in non-WASI module and no output detected".into())
}
