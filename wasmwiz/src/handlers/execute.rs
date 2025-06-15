// src/handlers/execute.rs
use actix_web::{web, HttpResponse, Result as ActixResult, HttpRequest, HttpMessage};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use tracing::{info, error, warn};
use std::time::Instant;

use crate::models::api_payloads::ExecuteResponse;
use crate::models::usage_log::UsageLog;
use crate::utils::file_system;
use crate::errors::ApiError;
use crate::middleware::auth::AuthContext;
use wasmer::{Store, Module};
use wasmer_wasix::{WasiEnv, Pipe};
use std::fs;
use std::time::Duration;
use tokio::time::timeout;

/// Execute a WebAssembly module with provided input
pub async fn execute_wasm(
    req: HttpRequest,
    app_state: web::Data<crate::AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse, ApiError> {
    let start_time = Instant::now();
    
    // Get authentication context
    let auth_context = req.extensions().get::<AuthContext>().cloned()
        .ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;
    
    info!("WASM execution request received for user: {}", auth_context.user.email);
    
    let mut wasm_data: Option<Vec<u8>> = None;
    let mut input_data: Option<String> = None;
    let mut wasm_size = 0;
    let mut input_size = 0;
    
    // Parse multipart form data
    while let Some(field) = payload.try_next().await.map_err(|e| {
        error!("Failed to parse multipart data: {}", e);
        ApiError::BadRequest("Failed to parse multipart data".to_string())
    })? {
        let field_name = field.name();
        
        match field_name {
            "wasm" => {
                let data = collect_field_data(field, app_state.config.max_wasm_size).await?;
                
                // Validate WASM magic bytes
                if !is_valid_wasm(&data) {
                    return Err(ApiError::BadRequest("Invalid WASM file format".to_string()));
                }
                
                wasm_size = data.len();
                wasm_data = Some(data);
            }
            "input" => {
                let data = collect_field_data(field, app_state.config.max_input_size).await?;
                input_size = data.len();
                input_data = Some(String::from_utf8(data).map_err(|_| {
                    ApiError::BadRequest("Input must be valid UTF-8".to_string())
                })?);
            }
            _ => {
                warn!("Unknown field in multipart data: {}", field_name);
            }
        }
    }
    
    // Validate required fields
    let wasm_data = wasm_data.ok_or_else(|| {
        ApiError::BadRequest("Missing 'wasm' field in form data".to_string())
    })?;
    
    let input_data = input_data.unwrap_or_default();
    
    // Save WASM to temporary file
    let temp_path = file_system::create_unique_wasm_file_path().await.map_err(|e| {
        error!("Failed to create temp file path: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Failed to create temporary file"))
    })?;
    
    tokio::fs::write(&temp_path, &wasm_data).await.map_err(|e| {
        error!("Failed to write WASM to temp file: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Failed to save WASM file"))
    })?;
    
    info!("WASM file saved to: {:?}", temp_path);
    
    // Execute WASM
    let result = execute_wasm_file(&temp_path, &input_data, &auth_context.tier, &app_state.config).await;
    
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
    data.len() >= 4 && &data[0..4] == &[0x00, 0x61, 0x73, 0x6D]
}

async fn execute_wasm_file(
    wasm_path: &std::path::Path,
    input: &str,
    tier: &crate::models::subscription_tier::SubscriptionTier,
    _config: &crate::config::Config,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Read the WASM file
    let wasm_bytes = fs::read(wasm_path)?;

    // Set up the Wasmer store (default engine)
    let mut store = Store::default();
    let module = Module::new(&store, &wasm_bytes)?;

    // Set up WASI environment
    let mut stdout_pipe = Pipe::new();
    let mut stdin_pipe = Pipe::new();
    use std::io::Write;
    stdin_pipe.write_all(input.as_bytes())?;
    let mut wasi_env = WasiEnv::builder("wasmwiz")
        .stdin(Box::new(stdin_pipe))
        .stdout(Box::new(stdout_pipe.clone()))
        .stderr(Box::new(Pipe::new()))
        .build()?;

    // Prepare for timeout
    let exec_timeout = Duration::from_secs(tier.max_execution_time_seconds as u64);
    let mut stdout_pipe_clone = stdout_pipe.clone();
    let module_clone = module.clone();

    let run_result = timeout(exec_timeout, tokio::task::spawn_blocking(move || {
        let mut store = Store::default();
        let import_object = wasmer::imports! {};
        let instance = wasmer::Instance::new(&mut store, &module_clone, &import_object)?;
        // Call the _start function (WASI entrypoint)
        let start = instance.exports.get_function("_start")?;
        start.call(&mut store, &[])?;
        use std::io::Read;
        let mut stdout = Vec::new();
        stdout_pipe_clone.read_to_end(&mut stdout)?;
        let output = String::from_utf8_lossy(&stdout).to_string();
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(output)
    })).await;

    match run_result {
        Ok(Ok(Ok(output))) => Ok(output),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(e)) => Err(Box::new(e)),
        Err(_) => Err("WASM execution timed out".into()),
    }
}
