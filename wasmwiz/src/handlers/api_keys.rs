// src/handlers/api_keys.rs
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use sha2::{Sha256, Digest};
use tracing::{info, error};

use crate::{
    models::{ApiKey, User},
    errors::ApiError,
    app::AppState,
};

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub user_email: String,
    pub tier_name: String,
}

#[derive(Serialize)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
    pub api_key_id: Uuid,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub key_hash: String, // Only show first 8 characters for security
    pub is_active: bool,
    pub created_at: String,
    pub tier_name: String,
}

/// Generate a new API key for a user
pub async fn create_api_key(
    app_state: web::Data<AppState>,
    req: web::Json<CreateApiKeyRequest>,
) -> ActixResult<HttpResponse, ApiError> {
    info!("Creating API key for user: {}", req.user_email);
    
    // Generate a secure random API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    
    // Find or create user
    let user = find_or_create_user(&app_state, &req.user_email).await?;
    
    // Find subscription tier
    let tier = find_tier_by_name(&app_state, &req.tier_name).await?;
    
    // Create API key record
    let api_key_record = ApiKey {
        id: Uuid::new_v4(),
        key_hash: key_hash.clone(),
        user_id: user.id,
        tier_id: tier.id,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Save to database
    app_state.db_service.create_api_key(&api_key_record).await
        .map_err(|e| {
            error!("Failed to create API key: {}", e);
            ApiError::InternalError(anyhow::anyhow!("Failed to create API key"))
        })?;
    
    info!("API key created successfully for user: {}", req.user_email);
    
    Ok(HttpResponse::Created().json(CreateApiKeyResponse {
        api_key,
        api_key_id: api_key_record.id,
        created_at: api_key_record.created_at.to_rfc3339(),
    }))
}

/// List API keys for a user
pub async fn list_api_keys(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse, ApiError> {
    let user_email = path.into_inner();
    info!("Listing API keys for user: {}", user_email);
    
    // Find user by email
    let user = find_user_by_email(&app_state, &user_email).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    // Get user's API keys
    let api_keys = app_state.db_service.get_user_api_keys(user.id).await
        .map_err(|e| {
            error!("Failed to fetch user API keys: {}", e);
            ApiError::InternalError(anyhow::anyhow!("Failed to fetch API keys"))
        })?;
    
    // Convert to response format
    let mut api_key_infos = Vec::new();
    for api_key in api_keys {
        // Get tier information
        if let Ok(Some(tier)) = app_state.db_service.find_subscription_tier_by_id(api_key.tier_id).await {
            api_key_infos.push(ApiKeyInfo {
                id: api_key.id,
                key_hash: format!("{}...", &api_key.key_hash[..8]), // Show only first 8 chars
                is_active: api_key.is_active,
                created_at: api_key.created_at.to_rfc3339(),
                tier_name: tier.name,
            });
        }
    }
    
    Ok(HttpResponse::Ok().json(api_key_infos))
}

/// Deactivate an API key
pub async fn deactivate_api_key(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse, ApiError> {
    let api_key_id = path.into_inner();
    info!("Deactivating API key: {}", api_key_id);
    
    app_state.db_service.deactivate_api_key(api_key_id).await
        .map_err(|e| {
            error!("Failed to deactivate API key: {}", e);
            ApiError::InternalError(anyhow::anyhow!("Failed to deactivate API key"))
        })?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "API key deactivated successfully"
    })))
}

// Helper functions

fn generate_api_key() -> String {
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    format!("ww_{}", key) // Prefix with "ww_" for WasmWiz
}

fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn find_or_create_user(
    app_state: &web::Data<AppState>,
    email: &str,
) -> Result<User, ApiError> {
    // Try to find existing user
    if let Ok(Some(user)) = find_user_by_email(app_state, email).await {
        return Ok(user);
    }
    
    // Create new user
    let new_user = User {
        id: Uuid::new_v4(),
        email: email.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Save to database
    sqlx::query(
        "INSERT INTO users (id, email, created_at, updated_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(new_user.id)
    .bind(&new_user.email)
    .bind(new_user.created_at)
    .bind(new_user.updated_at)
    .execute(&app_state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to create user: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Failed to create user"))
    })?;
    
    Ok(new_user)
}

async fn find_user_by_email(
    app_state: &web::Data<AppState>,
    email: &str,
) -> Result<Option<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(&app_state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to find user by email: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Database error"))
    })?;
    
    Ok(user)
}

async fn find_tier_by_name(
    app_state: &web::Data<AppState>,
    tier_name: &str,
) -> Result<crate::models::SubscriptionTier, ApiError> {
    let tier = sqlx::query_as::<_, crate::models::SubscriptionTier>(
        "SELECT * FROM subscription_tiers WHERE name = $1"
    )
    .bind(tier_name)
    .fetch_optional(&app_state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to find tier by name: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Database error"))
    })?
    .ok_or_else(|| ApiError::BadRequest(format!("Invalid tier name: {}", tier_name)))?;
    
    Ok(tier)
}
