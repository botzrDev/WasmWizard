//! # Health Check Handlers
//!
//! This module provides comprehensive health check endpoints for monitoring
//! the Wasm Wizard application status and diagnosing issues in production.
//!
//! ## Health Check Types
//!
//! ### Basic Health Check (`/health`)
//! - Database connectivity
//! - Filesystem access
//! - System resources (CPU, memory)
//! - Application responsiveness
//!
//! ### Kubernetes Liveness Probe (`/health/live`)
//! - Quick check for application liveness
//! - Used by Kubernetes to restart unhealthy pods
//!
//! ### Kubernetes Readiness Probe (`/health/ready`)
//! - Comprehensive check including dependencies
//! - Used by Kubernetes for load balancing
//!
//! ## Response Format
//!
//! ### Healthy Response
//! ```json
//! {
//!   "status": "healthy",
//!   "timestamp": "2024-01-01T12:00:00Z",
//!   "checks": {
//!     "database": {"status": "healthy", "message": "Connected"},
//!     "filesystem": {"status": "healthy", "message": "Writable"},
//!     "system": {"status": "healthy", "message": "CPU: 45%, Memory: 60%"}
//!   }
//! }
//! ```
//!
//! ### Unhealthy Response
//! ```json
//! {
//!   "status": "unhealthy",
//!   "timestamp": "2024-01-01T12:00:00Z",
//!   "checks": {
//!     "database": {"status": "unhealthy", "message": "Connection timeout"},
//!     "filesystem": {"status": "healthy", "message": "Writable"}
//!   }
//! }
//! ```
//!
//! ## Monitoring Integration
//!
//! Health check results are designed to integrate with:
//! - **Prometheus**: Metrics collection and alerting
//! - **Kubernetes**: Pod lifecycle management
//! - **Load Balancers**: Traffic routing decisions
//! - **Uptime Monitors**: External service monitoring

use crate::app::AppState;
use crate::log_info;
use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use sysinfo::System;
use tracing::error;

/// Comprehensive health check endpoint.
///
/// Performs detailed checks of all system components and dependencies.
/// This endpoint is used for monitoring the overall health of the Wasm Wizard
/// application and can be used by load balancers and monitoring systems.
///
/// # Checks Performed
///
/// - **Database**: Connection and query execution
/// - **Filesystem**: Read/write access to WASM temp directory
/// - **System Resources**: CPU and memory usage
/// - **Application**: General responsiveness
///
/// # Returns
///
/// - `200 OK`: All checks passed, system is healthy
/// - `503 Service Unavailable`: One or more checks failed
///
/// # Examples
///
/// ## Healthy System
/// ```json
/// {
///   "status": "healthy",
///   "timestamp": "2024-01-01T12:00:00Z",
///   "checks": {
///     "database": {"status": "healthy", "message": "Connected"},
///     "filesystem": {"status": "healthy", "message": "Writable"},
///     "system": {"status": "healthy", "message": "CPU: 25%, Memory: 40%"}
///   }
/// }
/// ```
///
/// ## Unhealthy System
/// ```json
/// {
///   "status": "unhealthy",
///   "timestamp": "2024-01-01T12:00:00Z",
///   "checks": {
///     "database": {"status": "unhealthy", "message": "Connection refused"},
///     "filesystem": {"status": "healthy", "message": "Writable"}
///   }
/// }
/// ```
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
        "service": "wasm-wizard",
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
    use prometheus::{gather, Encoder, TextEncoder};

    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(buffer))
}
