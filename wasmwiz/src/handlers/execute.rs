// src/handlers/execute.rs
use actix_multipart::Multipart;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Result as ActixResult, web};
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
use wasmer_wasix::{Pipe, WasiEnv};

use crate::app::AppState;
use crate::errors::ApiError;
use crate::middleware::auth::AuthContext;
use crate::models::api_payloads::ExecuteResponse;
use crate::models::usage_log::UsageLog;
use crate::utils::file_system;
use std::fs;

/// Execute a WebAssembly module with provided input
pub async fn execute_wasm(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse, ApiError> {
    let start_time = Instant::now();

    // Get authentication context
    let auth_context = req
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    info!("WASM execution request received for user: {}", auth_context.user.email);

    let mut wasm_data: Option<Vec<u8>> = None;
    let mut input_data: Option<String> = None;
    let mut wasm_size = 0;
    let mut input_size = 0;

    // Parse multipart form data
    info!("Starting multipart form parsing (authenticated)");
    let parse_timeout = Duration::from_secs(30);
    
    let parse_result = timeout(parse_timeout, async {
        while let Some(field) = payload.try_next().await.map_err(|e| {
            error!("Failed to parse multipart data: {}", e);
            ApiError::BadRequest("Failed to parse multipart data".to_string())
        })? {
            let field_name = field.name();
            info!("Processing multipart field: {}", field_name);

            match field_name {
                "wasm" => {
                    info!("Reading WASM file data");
                    let data = collect_field_data(field, app_state.config.max_wasm_size).await?;

                    // Validate WASM magic bytes
                    if !is_valid_wasm(&data) {
                        return Err(ApiError::BadRequest("Invalid WASM file format".to_string()));
                    }

                    wasm_size = data.len();
                    info!("WASM file read and validated successfully: {} bytes", wasm_size);
                    wasm_data = Some(data);
                }
                "input" => {
                    info!("Reading input data");
                    let data = collect_field_data(field, app_state.config.max_input_size).await?;
                    input_size = data.len();
                    input_data =
                        Some(String::from_utf8(data).map_err(|_| {
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
        Ok(Ok(())) => {
            info!("Multipart parsing completed successfully (authenticated)");
        }
        Ok(Err(e)) => {
            error!("Multipart parsing failed (authenticated): {}", e);
            return Err(e);
        }
        Err(_) => {
            error!("Multipart parsing timed out after {:?} (authenticated)", parse_timeout);
            return Err(ApiError::BadRequest("Multipart parsing timed out".to_string()));
        }
    }

    // Validate required fields
    let wasm_data = wasm_data
        .ok_or_else(|| ApiError::BadRequest("Missing 'wasm' field in form data".to_string()))?;

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

    // Execute WASM
    let result =
        execute_wasm_file(&temp_path, &input_data, &auth_context.tier, &app_state.config).await;

    // Calculate execution time
    let execution_time_ms = start_time.elapsed().as_millis() as i32;

    // Clean up temp file
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        warn!("Failed to clean up temp file {:?}: {}", temp_path, e);
    }

    // Create usage log and response
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
            let usage_log = UsageLog::error(auth_context.api_key.id, e.to_string())
                .with_execution_duration(execution_time_ms)
                .with_file_sizes(wasm_size as i32, input_size as i32);

            let response = HttpResponse::UnprocessableEntity().json(ExecuteResponse {
                output: None,
                error: Some(format!("Execution failed: {}", e)),
            });

            (response, usage_log)
        }
    };

    // Log usage (don't fail the request if logging fails)
    if let Err(e) = app_state.db_service.create_usage_log(&usage_log).await {
        error!("Failed to log usage: {}", e);
    }

    Ok(response)
}

/// Execute WASM without authentication (for development/demo mode)
pub async fn execute_wasm_no_auth(
    req: HttpRequest,
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
    use std::io::Write;

    // Read the WASM file
    let wasm_bytes = fs::read(wasm_path)?;

    // Set up the Wasmer store
    let mut store = Store::default();
    let module = Module::new(&store, &wasm_bytes)?;

    // Set up WASI environment with pipes for I/O
    let mut stdin_pipe = Pipe::new();
    stdin_pipe.write_all(input.as_bytes())?;
    let stdout_pipe = Pipe::new();
    let stderr_pipe = Pipe::new();

    use tracing::debug;
    debug!("WASM module imports: {:?}", module.imports().collect::<Vec<_>>());
    debug!("WASM module exports: {:?}", module.exports().collect::<Vec<_>>());

    // Try to build WASI environment with better error handling
    let mut wasi_env = WasiEnv::builder("wasmwiz")
        .stdin(Box::new(stdin_pipe))
        .stdout(Box::new(stdout_pipe.clone()))
        .stderr(Box::new(stderr_pipe))
        .finalize(&mut store)?;

    // Get import object for WASI, with improved error handling for version issues
    let import_object = match wasi_env.import_object(&mut store, &module) {
        Ok(obj) => {
            debug!("WASI import object created successfully");
            obj
        },
        Err(e) => {
            debug!("WASI import_object failed: {}. Attempting fallback approaches.", e);
            
            // Check if this is a non-WASI WASM file
            let has_wasi_imports = module.imports().any(|import| {
                import.module() == "wasi_snapshot_preview1" || 
                import.module() == "wasi_unstable" ||
                import.module().starts_with("wasi")
            });
            
            if !has_wasi_imports {
                debug!("Module appears to be a non-WASI WASM file, attempting direct execution");
                return execute_non_wasi_wasm(&mut store, &module, input, tier).await;
            }
            
            return Err(
                format!("WASI version could not be determined or is unsupported: {}. Module has WASI imports but version detection failed.", e).into()
            );
        }
    };

    // Create instance with improved error handling
    let instance = match Instance::new(&mut store, &module, &import_object) {
        Ok(instance) => {
            debug!("WASM instance created successfully");
            instance
        },
        Err(err) => {
            debug!("Instance creation failed: {}", err);
            return Err(format!("WASM instance creation failed: {}", err).into());
        }
    };

    // Initialize WASI environment with the instance
    wasi_env.initialize(&mut store, instance.clone())?;

    // Prepare for timeout execution
    let exec_timeout = Duration::from_secs(tier.max_execution_time_seconds as u64);

    let run_result = timeout(
        exec_timeout,
        tokio::task::spawn_blocking(move || {
            // Call the _start function (WASI entrypoint)
            if let Ok(start_func) = instance.exports.get_function("_start") {
                debug!("Calling _start function");
                let _ = start_func.call(&mut store, &[])?;
            } else {
                debug!("No _start function found, checking for main");
                if let Ok(main_func) = instance.exports.get_function("main") {
                    debug!("Calling main function");
                    let _ = main_func.call(&mut store, &[])?;
                } else {
                    return Err("No suitable entry point found (_start or main)".into());
                }
            }

            // Read stdout content
            use std::io::Read;
            let mut stdout_content = Vec::new();
            let mut stdout_pipe_clone = stdout_pipe.clone();
            stdout_pipe_clone.read_to_end(&mut stdout_content)?;
            let output = String::from_utf8_lossy(&stdout_content).to_string();

            Ok::<String, Box<dyn std::error::Error + Send + Sync>>(output)
        }),
    )
    .await;

    match run_result {
        Ok(join_result) => match join_result {
            Ok(wasm_result) => wasm_result, // This is already Result<String, Box<dyn Error>>
            Err(join_error) => Err(format!("Task join error: {}", join_error).into()),
        },
        Err(_) => Err("WASM execution timed out".into()),
    }
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
    
    // Create a simple import object for basic functionality
    let import_object = imports! {
        "env" => {
            // Add basic environment functions if needed
        }
    };
    
    let instance = Instance::new(store, module, &import_object)?;
    
    // Look for exported functions to call
    let exports = instance.exports;
    
    // Try common function names
    let function_names = ["main", "run", "execute", "start", "_start"];
    
    for func_name in &function_names {
        if let Ok(func) = exports.get_function(func_name) {
            debug!("Found and calling function: {}", func_name);
            
            let exec_timeout = Duration::from_secs(tier.max_execution_time_seconds as u64);
            
            let result = timeout(
                exec_timeout,
                tokio::task::spawn_blocking({
                    let func = func.clone();
                    move || {
                        let mut local_store = Store::default();
                        // Call function with no parameters for now
                        let result = func.call(&mut local_store, &[])?;
                        
                        // Convert result to string if possible
                        if result.is_empty() {
                            Ok("Function executed successfully (no return value)".to_string())
                        } else {
                            Ok(format!("Function returned: {:?}", result))
                        }
                    }
                })
            ).await;
            
            return match result {
                Ok(join_result) => match join_result {
                    Ok(output) => output,
                    Err(e) => Err(format!("Function execution error: {}", e).into()),
                },
                Err(_) => Err("Function execution timed out".into()),
            };
        }
    }
    
    Err("No suitable entry point found in non-WASI module".into())
}
