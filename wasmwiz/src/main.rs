//! # Wasm Wizard Application Server
//!
//! This is the main entry point for the Wasm Wizard WebAssembly execution platform.
//! It sets up the Actix-web server with all necessary middleware, routes, and services.
//!
//! ## Startup Process
//!
//! The application follows a structured startup sequence:
//!
//! 1. **Environment Setup**: Load environment variables from `.env` file (development only)
//! 2. **Configuration**: Load and validate application configuration
//! 3. **Logging**: Initialize structured logging with appropriate verbosity
//! 4. **Database**: Establish connection pool and run migrations (development)
//! 5. **Background Tasks**: Start cleanup tasks for temporary files
//! 6. **Server**: Configure and start the HTTP server
//!
//! ## Configuration
//!
//! The application is configured entirely through environment variables.
//! See [`config::Config`] for complete configuration options.
//!
//! ## Security Features
//!
//! - **Environment-based config**: No hardcoded secrets or credentials
//! - **Validation**: Configuration is validated on startup
//! - **Sandboxing**: WASM execution is isolated with resource limits
//! - **Authentication**: API key-based authentication (configurable)
//! - **Rate Limiting**: Distributed rate limiting with Redis support
//!
//! ## Monitoring
//!
//! The application provides comprehensive monitoring:
//!
//! - **Health Checks**: `/health`, `/health/live`, `/health/ready`
//! - **Metrics**: `/metrics` (Prometheus format)
//! - **Structured Logging**: JSON-formatted logs with correlation IDs
//! - **Performance Monitoring**: Request timing and resource usage
//!
//! ## Example
//!
//! ```bash
//! # Development
//! cargo run
//!
//! # Production
//! export ENVIRONMENT=production
//! export DATABASE_URL="postgresql://..."
//! export API_SALT="your-secure-salt"
//! cargo run --release
//! ```

mod app;
mod config;
mod errors;
mod handlers;
mod logging;
mod middleware;
mod models;
mod services;
mod utils;

use std::sync::Arc;

use actix_web::HttpServer;
use anyhow::{Context, Result};
use dotenvy::dotenv;
use tracing::{error, info};

use app::create_app;
use config::Config;
use logging::init_logging;
use services::establish_connection_pool;
use services::{cleanup, DatabaseService};
use utils::file_system;

#[actix_web::main] // Marks the main function as the Actix-web entry point
async fn main() -> Result<()> {
    // 1. Load environment variables from .env file (for local development)
    if cfg!(debug_assertions) {
        dotenv().ok(); // Only load .env in development builds
    }

    // 2. Load and validate configuration
    let config = Config::from_env().context("failed to load application configuration")?;
    config
        .validate()
        .context("configuration validation failed")?;
    let config = Arc::new(config);

    // 3. Initialize logging based on environment
    init_logging(&config)
        .map_err(|e| anyhow::anyhow!("failed to initialise structured logging: {}", e))?;

    info!(
        environment = ?config.environment,
        version = env!("CARGO_PKG_VERSION"),
        "Wasm Wizard starting up"
    );

    // 4. Database connection and setup
    info!("Connecting to database: {}", &config.database_url);
    let db_pool = establish_connection_pool(&config)
        .await
        .context("failed to create database connection pool")?;
    info!("Database connection pool established");

    // 5. Run database migrations (auto-run in development, manual in production)
    if config.is_development() {
        info!("Development mode: Running database migrations...");
        sqlx::migrate!("./migrations") // Path to your migrations directory
            .run(&db_pool)
            .await
            .map_err(|error| {
                error!("Failed to run database migrations: {:?}", error);
                error
            })?;
        info!("Database migrations complete");
    } else {
        info!("Production/Staging mode: Migrations should be run manually before deployment");
    }

    // 6. Start background cleanup tasks
    file_system::start_wasm_cleanup_task();
    cleanup::start_cleanup_tasks(DatabaseService::new(db_pool.clone()));
    info!("Background cleanup tasks started");

    // 7. Set up Actix-web server with production optimizations
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    let is_production = config.is_production();

    info!(
        host = %server_host,
        port = %server_port,
        "Starting Actix-web server"
    );

    let config_for_server = Arc::clone(&config);
    let server =
        HttpServer::new(move || create_app(db_pool.clone(), Arc::clone(&config_for_server)))
            .bind((server_host.as_str(), server_port))
            .with_context(|| format!("failed to bind server to {}:{}", server_host, server_port))?;

    // Production server settings
    let server = if is_production {
        server
            .workers(num_cpus::get()) // Use all available CPU cores
            .keep_alive(std::time::Duration::from_secs(75)) // Keep connections alive
            .client_request_timeout(std::time::Duration::from_secs(30))
            .client_disconnect_timeout(std::time::Duration::from_secs(5))
    } else {
        server.workers(1) // Single worker for development
    };

    server
        .run()
        .await
        .context("Actix-web server terminated unexpectedly")?;

    info!("Server shut down gracefully");
    Ok(())
}
