// src/handlers/execute.rs
use actix_web::{web, HttpResponse, Result as ActixResult, HttpRequest};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::io::Write;
use tracing::{info, error, warn};
use uuid::Uuid;
use std::time::Instant;

use crate::models::api_payloads::ExecuteResponse;
use crate::models::usage_log::UsageLog;
use crate::utils::file_system;
use crate::errors::ApiError;
use crate::middleware::auth::AuthContext;
use crate::services::DatabaseService;

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
        let field_name = field.name().unwrap_or("");
        
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
    config: &crate::config::Config,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use wasmer::{Store, Module, Instance, imports};
    use wasmer_wasix::{WasiState, WasiError};
    use std::time::Duration;
    
    info!("Starting WASM execution");
    
    // Create a WASM store
    let mut store = Store::default();
    
    // Read and compile the WASM module
    let wasm_bytes = tokio::fs::read(wasm_path).await?;
    let module = Module::new(&store, wasm_bytes)?;
    
    // Set up WASI with limited capabilities
    let mut wasi_state_builder = WasiState::new("wasm_module");
    
    // Provide input via stdin
    if !input.is_empty() {
        wasi_state_builder = wasi_state_builder.stdin(Box::new(std::io::Cursor::new(input.as_bytes())));
    }
    
    // Capture stdout
    let stdout_buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stdout_clone = stdout_buffer.clone();
    wasi_state_builder = wasi_state_builder.stdout(Box::new(SharedBuffer::new(stdout_clone)));
    
    // Build WASI state with restrictions
    let wasi_state = wasi_state_builder
        .env("WASM_EXECUTION", "1") // Minimal env for identification
        .finalize(&mut store)?;
    
    // Get WASI imports
    let import_object = wasi_state.import_object(&mut store, &module)?;
    
    // Instantiate the module
    let instance = Instance::new(&mut store, &module, &import_object)?;
    
    // Set up execution timeout based on tier limits
    let timeout_duration = std::time::Duration::from_secs(
        std::cmp::min(tier.max_execution_time_seconds as u64, config.execution_timeout)
    );
    let start_time = std::time::Instant::now();
    
    // Execute with timeout
    let result = tokio::time::timeout(timeout_duration, async {
        // Get the start function
        if let Ok(start_func) = instance.exports.get_function("_start") {
            start_func.call(&mut store, &[])?;
        } else {
            return Err("WASM module does not have a _start function".into());
        }
        
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }).await;
    
    let execution_time = start_time.elapsed();
    info!("WASM execution completed in {:?}", execution_time);
    
    match result {
        Ok(Ok(())) => {
            // Get output from stdout buffer
            let output = {
                let buffer = stdout_buffer.lock().unwrap();
                String::from_utf8_lossy(&buffer).to_string()
            };
            Ok(output)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("WASM execution timed out".into()),
    }
}

// Helper struct for capturing stdout
#[derive(Clone)]
struct SharedBuffer {
    buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

impl SharedBuffer {
    fn new(buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>) -> Self {
        Self { buffer }
    }
}

impl Write for SharedBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
