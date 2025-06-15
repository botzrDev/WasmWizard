// src/handlers/health.rs
use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info};

/// Health check endpoint that verifies system components
pub async fn health_check(pool: web::Data<crate::AppState>) -> Result<HttpResponse> {
    info!("Health check requested");
    
    let mut status = "healthy";
    let mut checks = serde_json::Map::new();
    
    // Check database connectivity
    match sqlx::query("SELECT 1").fetch_one(&pool.db_pool).await {
        Ok(_) => {
            checks.insert("database".to_string(), json!({"status": "healthy", "message": "Connected"}));
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            checks.insert("database".to_string(), json!({"status": "unhealthy", "message": format!("Database error: {}", e)}));
            status = "unhealthy";
        }
    }
    
    // Check filesystem access for WASM temp directory
    let temp_dir = std::env::var("WASM_TEMP_DIR").unwrap_or_else(|_| "/tmp/wasm_modules".to_string());
    match tokio::fs::create_dir_all(&temp_dir).await {
        Ok(_) => {
            checks.insert("filesystem".to_string(), json!({"status": "healthy", "message": "Writable"}));
        }
        Err(e) => {
            error!("Filesystem health check failed: {}", e);
            checks.insert("filesystem".to_string(), json!({"status": "unhealthy", "message": format!("Filesystem error: {}", e)}));
            status = "unhealthy";
        }
    }
    
    let response = json!({
        "status": status,
        "timestamp": chrono::Utc::now(),
        "service": "wasmwiz",
        "version": env!("CARGO_PKG_VERSION"),
        "checks": checks
    });
    
    if status == "healthy" {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}
