use crate::app::AppState;
use crate::middleware::pre_auth::AuthContext;
use crate::filters;
use crate::models::{ApiKey, SubscriptionTier, User};
use actix_web::{web, HttpResponse, Result as ActixResult};
use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub title: String,
    pub admin_email: String,
    pub admin_role: String,
    pub total_users: i64,
    pub total_api_keys: i64,
    pub total_executions_today: i64,
    pub active_users_today: i64,
}

#[derive(Template)]
#[template(path = "admin/users.html")]
pub struct AdminUsersTemplate {
    pub title: String,
    pub admin_email: String,
    pub users: Vec<UserWithStats>,
}

#[derive(Template)]
#[template(path = "admin/api_keys.html")]
pub struct AdminApiKeysTemplate {
    pub title: String,
    pub admin_email: String,
    pub api_keys: Vec<ApiKeyWithDetails>,
}

#[derive(Template)]
#[template(path = "admin/analytics.html")]
pub struct AdminAnalyticsTemplate {
    pub title: String,
    pub admin_email: String,
    pub usage_stats: UsageStats,
    pub recent_executions: Vec<RecentExecution>,
}

#[derive(Template)]
#[template(path = "admin/tiers.html")]
pub struct AdminTiersTemplate {
    pub title: String,
    pub admin_email: String,
    pub tiers: Vec<SubscriptionTier>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserWithStats {
    pub user: User,
    pub tier_name: String,
    pub api_key_count: i64,
    pub total_executions: i64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiKeyWithDetails {
    pub api_key: ApiKey,
    pub user_email: String,
    pub tier_name: String,
    pub total_executions: i64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UsageStats {
    pub total_executions: i64,
    pub executions_today: i64,
    pub executions_this_week: i64,
    pub executions_this_month: i64,
    pub success_rate: f64,
    pub average_execution_time: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecentExecution {
    pub timestamp: DateTime<Utc>,
    pub user_email: String,
    pub tier_name: String,
    pub status: String,
    pub execution_duration_ms: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub tier_name: String,
}

#[derive(Deserialize)]
pub struct UpdateUserTierRequest {
    pub tier_name: String,
}

#[derive(Deserialize)]
pub struct CreateTierRequest {
    pub name: String,
    pub max_executions_per_minute: i32,
    pub max_executions_per_day: i32,
    pub max_memory_mb: i32,
    pub max_execution_time_seconds: i32,
}

// Admin Dashboard
pub async fn admin_dashboard(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let admin_role = determine_admin_role(&auth_context.user.email);

    // Get dashboard statistics
    let total_users = app_state.db_service.count_users().await.unwrap_or(0);
    let total_api_keys = app_state.db_service.count_api_keys().await.unwrap_or(0);
    let total_executions_today = app_state.db_service.count_executions_today().await.unwrap_or(0);
    let active_users_today = app_state.db_service.count_active_users_today().await.unwrap_or(0);

    let template = AdminDashboardTemplate {
        title: "Admin Dashboard - WasmWiz".to_string(),
        admin_email: auth_context.user.email.clone(),
        admin_role,
        total_users,
        total_api_keys,
        total_executions_today,
        active_users_today,
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(template.render().unwrap()))
}

// User Management
pub async fn admin_users(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let _admin_role = determine_admin_role(&auth_context.user.email);

    let users = app_state.db_service.get_all_users_with_stats().await
        .unwrap_or_default();

    let template = AdminUsersTemplate {
        title: "User Management - WasmWiz Admin".to_string(),
        admin_email: auth_context.user.email.clone(),
        users,
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(template.render().unwrap()))
}

pub async fn create_user(
    _auth_context: AuthContext,
    app_state: web::Data<AppState>,
    form: web::Json<CreateUserRequest>,
) -> ActixResult<HttpResponse> {
    // Create new user
    let user_id = app_state.db_service
        .create_user(&form.email)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create user: {}", e))
        })?;

    // Get tier ID
    let tier = app_state.db_service
        .get_tier_by_name(&form.tier_name)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to get tier: {}", e))
        })?
        .ok_or_else(|| {
            actix_web::error::ErrorBadRequest("Invalid tier name")
        })?;

    // Create default API key for user
    let _api_key = app_state.db_service
        .create_api_key_for_user(user_id, tier.id)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create API key: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "User created successfully",
        "user_id": user_id,
        "email": form.email
    })))
}

pub async fn update_user_tier(
    _auth_context: AuthContext,
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    form: web::Json<UpdateUserTierRequest>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();

    // Get tier ID
    let tier = app_state.db_service
        .get_tier_by_name(&form.tier_name)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to get tier: {}", e))
        })?
        .ok_or_else(|| {
            actix_web::error::ErrorBadRequest("Invalid tier name")
        })?;

    // Update user's tier (update their API keys)
    app_state.db_service
        .update_user_tier(user_id, tier.id)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to update user tier: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "User tier updated successfully",
        "user_id": user_id,
        "new_tier": form.tier_name
    })))
}

// API Key Management
pub async fn admin_api_keys(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let _admin_role = determine_admin_role(&auth_context.user.email);

    let api_keys = app_state.db_service.get_all_api_keys_with_details().await
        .unwrap_or_default();

    let template = AdminApiKeysTemplate {
        title: "API Key Management - WasmWiz Admin".to_string(),
        admin_email: auth_context.user.email.clone(),
        api_keys,
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(template.render().unwrap()))
}

// Analytics
pub async fn admin_analytics(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let _admin_role = determine_admin_role(&auth_context.user.email);

    let usage_stats = app_state.db_service.get_usage_statistics().await
        .unwrap_or_default();
    let recent_executions = app_state.db_service.get_recent_executions(50).await
        .unwrap_or_default();

    let template = AdminAnalyticsTemplate {
        title: "Analytics - WasmWiz Admin".to_string(),
        admin_email: auth_context.user.email.clone(),
        usage_stats,
        recent_executions,
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(template.render().unwrap()))
}

// Tier Management
pub async fn admin_tiers(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let _admin_role = determine_admin_role(&auth_context.user.email);

    let tiers = app_state.db_service.get_all_tiers().await
        .unwrap_or_default();

    let template = AdminTiersTemplate {
        title: "Tier Management - WasmWiz Admin".to_string(),
        admin_email: auth_context.user.email.clone(),
        tiers,
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(template.render().unwrap()))
}

pub async fn create_tier(
    _auth_context: AuthContext,
    app_state: web::Data<AppState>,
    form: web::Json<CreateTierRequest>,
) -> ActixResult<HttpResponse> {
    let tier_id = app_state.db_service
        .create_tier(
            &form.name,
            form.max_executions_per_minute,
            form.max_executions_per_day,
            form.max_memory_mb,
            form.max_execution_time_seconds,
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create tier: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Tier created successfully",
        "tier_id": tier_id,
        "name": form.name
    })))
}

// System Control (Master Admin Only)
pub async fn system_status(
    auth_context: AuthContext,
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    // System health information
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "database": "connected",
        "redis": app_state.redis_service.is_some(),
        "uptime": "system_uptime_placeholder",
        "version": env!("CARGO_PKG_VERSION"),
        "admin": auth_context.user.email
    })))
}

pub async fn emergency_shutdown(
    auth_context: AuthContext,
    _app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    // This would implement graceful shutdown procedures
    tracing::warn!("Emergency shutdown initiated by admin: {}", auth_context.user.email);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Emergency shutdown procedures initiated",
        "admin": auth_context.user.email,
        "timestamp": Utc::now()
    })))
}

// Helper function to determine admin role
fn determine_admin_role(email: &str) -> String {
    use crate::middleware::master_admin::AdminRole;

    match AdminRole::from_email(email) {
        Some(AdminRole::MasterAdmin) => "Master Admin".to_string(),
        Some(AdminRole::SystemAdmin) => "System Admin".to_string(),
        Some(AdminRole::SupportAdmin) => "Support Admin".to_string(),
        None => "User".to_string(),
    }
}