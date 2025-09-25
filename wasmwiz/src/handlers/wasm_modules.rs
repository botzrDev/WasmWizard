//! # WASM Module Management Handlers
//!
//! This module provides HTTP handlers for managing WebAssembly modules:
//! - Upload new WASM modules
//! - List user's modules (with pagination)
//! - Delete modules
//! - Get module metadata
//!
//! ## Security Features
//!
//! - All endpoints require API key authentication
//! - Users can only access their own modules (unless public)
//! - File size limits enforced
//! - SHA-256 integrity verification
//! - Input validation and sanitization

use crate::app::AppState;
use crate::errors::ApiError;
use crate::middleware::pre_auth::AuthContext;
use crate::models::{UploadModuleRequest, UploadModuleResponse, WasmModule, WasmModuleMeta};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Result as ActixResult};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListModulesQuery {
    /// Page number for pagination (1-based)
    pub page: Option<u32>,
    /// Number of results per page (max 100)
    pub limit: Option<u32>,
    /// Include public modules from other users
    pub include_public: Option<bool>,
}

#[derive(Serialize)]
pub struct ListModulesResponse {
    pub modules: Vec<WasmModuleMeta>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub has_next: bool,
}

/// Upload a new WASM module
///
/// Accepts a multipart form with:
/// - `wasm`: WASM binary file (required)
/// - `name`: Module name (required)
/// - `description`: Module description (optional)
/// - `is_public`: Whether module should be public (optional, default: false)
pub async fn upload_module(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse, ApiError> {
    let mut wasm_data: Option<Vec<u8>> = None;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public = false;

    // Process multipart form data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "wasm" => {
                let mut bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| {
                        error!("Failed to read WASM chunk: {}", e);
                        ApiError::BadRequest("Failed to read WASM data".to_string())
                    })?;
                    bytes.extend_from_slice(&data);

                    // Check size limit during upload to prevent memory exhaustion
                    if bytes.len() > app_state.config.max_wasm_size {
                        return Err(ApiError::PayloadTooLarge("WASM module too large".to_string()));
                    }
                }

                if bytes.is_empty() {
                    return Err(ApiError::BadRequest("WASM data is empty".to_string()));
                }

                // Validate WASM format (basic check)
                if !is_valid_wasm(&bytes) {
                    return Err(ApiError::BadRequest("Invalid WASM format".to_string()));
                }

                wasm_data = Some(bytes);
            }
            "name" => {
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk
                        .map_err(|_| ApiError::BadRequest("Failed to read name".to_string()))?;
                    field_data.extend_from_slice(&data);
                }
                name = Some(
                    String::from_utf8(field_data)
                        .map_err(|_| ApiError::BadRequest("Invalid UTF-8 in name".to_string()))?
                        .trim()
                        .to_string(),
                );
            }
            "description" => {
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|_| {
                        ApiError::BadRequest("Failed to read description".to_string())
                    })?;
                    field_data.extend_from_slice(&data);
                }
                let desc = String::from_utf8(field_data)
                    .map_err(|_| ApiError::BadRequest("Invalid UTF-8 in description".to_string()))?
                    .trim()
                    .to_string();
                if !desc.is_empty() {
                    description = Some(desc);
                }
            }
            "is_public" => {
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|_| {
                        ApiError::BadRequest("Failed to read is_public".to_string())
                    })?;
                    field_data.extend_from_slice(&data);
                }
                let is_public_str = String::from_utf8(field_data)
                    .map_err(|_| ApiError::BadRequest("Invalid UTF-8 in is_public".to_string()))?;
                is_public = is_public_str.trim() == "true";
            }
            _ => {
                // Skip unknown fields
                warn!("Unknown field in upload: {}", field_name);
            }
        }
    }

    // Validate required fields
    let wasm_data =
        wasm_data.ok_or_else(|| ApiError::BadRequest("WASM file is required".to_string()))?;
    let name = name.ok_or_else(|| ApiError::BadRequest("Module name is required".to_string()))?;

    // Validate name length and characters
    if name.is_empty() || name.len() > 255 {
        return Err(ApiError::BadRequest("Module name must be 1-255 characters".to_string()));
    }

    // Calculate SHA-256 hash for integrity and deduplication
    let mut hasher = Sha256::new();
    hasher.update(&wasm_data);
    let sha256_hash = format!("{:x}", hasher.finalize());

    // Create the WASM module record
    let module_id = Uuid::new_v4();
    let now = Utc::now();

    let wasm_module = WasmModule {
        id: module_id,
        name: name.clone(),
        description: description.clone(),
        user_id: auth_context.user.id,
        wasm_data: wasm_data.clone(),
        size_bytes: wasm_data.len() as i32,
        sha256_hash: sha256_hash.clone(),
        upload_time: now,
        last_executed: None,
        execution_count: 0,
        is_public,
        created_at: now,
        updated_at: now,
    };

    // Store in database
    let result = sqlx::query!(
        r#"
        INSERT INTO wasm_modules (
            id, name, description, user_id, wasm_data, size_bytes, 
            sha256_hash, upload_time, last_executed, execution_count, 
            is_public, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
        module_id,
        name,
        description,
        auth_context.user.id,
        wasm_data,
        wasm_module.size_bytes,
        sha256_hash,
        now,
        wasm_module.last_executed,
        wasm_module.execution_count,
        is_public,
        now,
        now
    )
    .execute(&app_state.db_pool)
    .await;

    match result {
        Ok(_) => {
            info!(
                "Module uploaded successfully: {} ({}MB) by user {}",
                name,
                wasm_module.size_bytes as f64 / 1_048_576.0,
                auth_context.user.email
            );

            let response = UploadModuleResponse {
                id: module_id,
                name,
                size_bytes: wasm_module.size_bytes,
                sha256_hash,
                upload_time: now,
            };

            Ok(HttpResponse::Created().json(response))
        }
        Err(sqlx::Error::Database(db_err))
            if db_err.constraint() == Some("wasm_modules_sha256_hash_key") =>
        {
            Err(ApiError::Conflict("Module with this content already exists".to_string()))
        }
        Err(e) => {
            error!("Database error storing WASM module: {}", e);
            Err(ApiError::DatabaseError("Failed to store module".to_string()))
        }
    }
}

/// List user's WASM modules with pagination
pub async fn list_modules(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
    query: web::Query<ListModulesQuery>,
) -> ActixResult<HttpResponse, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let include_public = query.include_public.unwrap_or(false);
    let offset = (page - 1) * limit;

    // Build query based on whether to include public modules
    let (modules, total) = if include_public {
        let modules = sqlx::query_as!(
            WasmModuleMeta,
            r#"
            SELECT id, name, description, size_bytes, sha256_hash,
                   upload_time, last_executed, execution_count, is_public
            FROM wasm_modules 
            WHERE user_id = $1 OR is_public = true
            ORDER BY upload_time DESC
            LIMIT $2 OFFSET $3
            "#,
            auth_context.user.id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error listing modules: {}", e);
            ApiError::DatabaseError("Failed to list modules".to_string())
        })?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM wasm_modules WHERE user_id = $1 OR is_public = true",
            auth_context.user.id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error counting modules: {}", e);
            ApiError::DatabaseError("Failed to count modules".to_string())
        })?
        .unwrap_or(0);

        (modules, total as u64)
    } else {
        let modules = sqlx::query_as!(
            WasmModuleMeta,
            r#"
            SELECT id, name, description, size_bytes, sha256_hash,
                   upload_time, last_executed, execution_count, is_public
            FROM wasm_modules 
            WHERE user_id = $1
            ORDER BY upload_time DESC
            LIMIT $2 OFFSET $3
            "#,
            auth_context.user.id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error listing user modules: {}", e);
            ApiError::DatabaseError("Failed to list modules".to_string())
        })?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM wasm_modules WHERE user_id = $1",
            auth_context.user.id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error counting user modules: {}", e);
            ApiError::DatabaseError("Failed to count modules".to_string())
        })?
        .unwrap_or(0);

        (modules, total as u64)
    };

    let has_next = (offset + limit) < total as u32;

    let response = ListModulesResponse {
        modules,
        page,
        limit,
        total,
        has_next,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete a WASM module
pub async fn delete_module(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse, ApiError> {
    let module_id = path.into_inner();

    // Check if module exists and user owns it
    let module =
        sqlx::query!("SELECT id, user_id, name FROM wasm_modules WHERE id = $1", module_id)
            .fetch_optional(&app_state.db_pool)
            .await
            .map_err(|e| {
                error!("Database error checking module ownership: {}", e);
                ApiError::DatabaseError("Failed to check module".to_string())
            })?;

    let module = module.ok_or_else(|| ApiError::NotFound("Module not found".to_string()))?;

    // Verify ownership
    if module.user_id != auth_context.user.id {
        return Err(ApiError::Forbidden("You can only delete your own modules".to_string()));
    }

    // Delete the module
    let result = sqlx::query!("DELETE FROM wasm_modules WHERE id = $1", module_id)
        .execute(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error deleting module: {}", e);
            ApiError::DatabaseError("Failed to delete module".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Module not found".to_string()));
    }

    info!(
        "Module deleted: {} (ID: {}) by user {}",
        module.name, module_id, auth_context.user.email
    );

    Ok(HttpResponse::NoContent().finish())
}

/// Basic WASM format validation
fn is_valid_wasm(data: &[u8]) -> bool {
    // Check for WASM magic number (0x00 0x61 0x73 0x6D)
    if data.len() < 8 {
        return false;
    }

    let magic = &data[0..4];
    let version = &data[4..8];

    // WASM magic number: \0asm
    magic == [0x00, 0x61, 0x73, 0x6D] &&
    // WASM version 1
    version == [0x01, 0x00, 0x00, 0x00]
}
