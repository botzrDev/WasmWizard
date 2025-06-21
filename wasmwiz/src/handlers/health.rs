// src/handlers/health.rs
use crate::app::AppState;
use crate::log_info;
use actix_web::{HttpResponse, Result, web};
use serde_json::json;
use sysinfo::System;
use tracing::error;

/// Health check endpoint that verifies system components
pub async fn health_check(pool: web::Data<AppState>) -> Result<HttpResponse> {
    log_info!("Health check requested");

    let mut status = "healthy";
    let mut checks = serde_json::Map::new();

    // Check database connectivity
    match sqlx::query("SELECT 1").fetch_one(&pool.db_pool).await {
        Ok(_) => {
            checks.insert(
                "database".to_string(),
                json!({"status": "healthy", "message": "Connected"}),
            );
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            status = "unhealthy";
            checks.insert(
                "database".to_string(),
                json!({"status": "unhealthy", "message": format!("Database error: {}", e)}),
            );
        }
    }

    // Check filesystem access for WASM temp directory
    let temp_dir =
        std::env::var("WASM_TEMP_DIR").unwrap_or_else(|_| "/tmp/wasm_modules".to_string());
    match tokio::fs::create_dir_all(&temp_dir).await {
        Ok(_) => {
            checks.insert(
                "filesystem".to_string(),
                json!({"status": "healthy", "message": "Writable"}),
            );
        }
        Err(e) => {
            error!("Filesystem health check failed: {}", e);
            checks.insert(
                "filesystem".to_string(),
                json!({"status": "unhealthy", "message": format!("Filesystem error: {}", e)}),
            );
            status = "unhealthy";
        }
    }

    // Resource utilization
    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = sysinfo::get_current_pid().unwrap();
    if let Some(proc) = sys.process(pid) {
        checks.insert("memory_mb".to_string(), json!(proc.memory() / 1024));
        checks.insert("cpu_usage".to_string(), json!(proc.cpu_usage()));
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

/// Liveness probe
pub async fn liveness_probe() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({"status": "alive"})))
}

/// Readiness probe with dependency and resource checks
pub async fn readiness_probe(pool: web::Data<AppState>) -> Result<HttpResponse> {
    let mut status = "ready";
    let mut checks = serde_json::Map::new();

    // Database check
    match sqlx::query("SELECT 1").fetch_one(&pool.db_pool).await {
        Ok(_) => {
            checks.insert("database".to_string(), json!({"status": "healthy"}));
        }
        Err(e) => {
            checks.insert(
                "database".to_string(),
                json!({"status": "unhealthy", "error": e.to_string()}),
            );
            status = "not_ready";
        }
    }

    // Redis check (if used)
    // checks.insert("redis".to_string(), json!({"status": "healthy"}));

    // Resource utilization
    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = sysinfo::get_current_pid().unwrap();
    if let Some(proc) = sys.process(pid) {
        checks.insert("memory_mb".to_string(), json!(proc.memory() / 1024));
        checks.insert("cpu_usage".to_string(), json!(proc.cpu_usage()));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": status,
        "checks": checks
    })))
}

/// Prometheus metrics endpoint
pub async fn prometheus_metrics() -> Result<HttpResponse> {
    use prometheus::{Encoder, TextEncoder, gather};

    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(buffer))
}
