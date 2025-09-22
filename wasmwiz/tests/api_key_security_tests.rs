use std::sync::Arc;

use actix_web::web;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use wasm_wizard::{
    app::AppState,
    config::Config,
    errors::ApiError,
    handlers::api_keys::{create_api_key, CreateApiKeyRequest},
    middleware::pre_auth::AuthContext,
    models::{ApiKey, SubscriptionTier, User},
    services::DatabaseService,
};

fn build_test_state() -> web::Data<AppState> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://postgres:postgres@localhost/test")
        .expect("failed to create lazy test pool");

    let db_service = DatabaseService::new(pool.clone());
    let config = Arc::new(Config::default());

    web::Data::new(AppState {
        db_pool: pool,
        db_service,
        config,
        redis_service: None,
    })
}

fn build_auth_context(email: &str, tier_name: &str) -> AuthContext {
    let user_id = Uuid::new_v4();
    let tier_id = Uuid::new_v4();

    AuthContext {
        api_key: ApiKey {
            id: Uuid::new_v4(),
            key_hash: "test_hash".to_string(),
            user_id,
            tier_id,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        user: User {
            id: user_id,
            email: email.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        tier: SubscriptionTier {
            id: tier_id,
            name: tier_name.to_string(),
            max_executions_per_minute: 10,
            max_executions_per_day: 100,
            max_memory_mb: 128,
            max_execution_time_seconds: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    }
}

#[actix_web::test]
async fn non_admin_cannot_create_privileged_api_key() {
    let app_state = build_test_state();
    let auth_context = build_auth_context("user@example.com", "Free");
    let request = web::Json(CreateApiKeyRequest {
        user_email: "admin@wasm-wizard.dev".to_string(),
        tier_name: "Free".to_string(),
    });

    let result = create_api_key(app_state.clone(), auth_context, request).await;

    match result {
        Err(ApiError::Forbidden(message)) => {
            assert!(message.to_lowercase().contains("administrator"));
        }
        other => panic!("expected forbidden error, got {:?}", other),
    }
}

#[actix_web::test]
async fn non_admin_cannot_create_keys_for_other_users() {
    let app_state = build_test_state();
    let auth_context = build_auth_context("user@example.com", "Free");
    let request = web::Json(CreateApiKeyRequest {
        user_email: "other@example.com".to_string(),
        tier_name: "Free".to_string(),
    });

    let result = create_api_key(app_state.clone(), auth_context, request).await;

    match result {
        Err(ApiError::Forbidden(message)) => {
            assert!(message.to_lowercase().contains("permission"));
        }
        other => panic!("expected forbidden error, got {:?}", other),
    }
}
