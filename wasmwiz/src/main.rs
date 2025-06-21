mod app;
mod config;
mod errors;
mod handlers;
mod logging;
mod middleware;
mod models;
mod services;
mod utils;

use actix_web::HttpServer;
use dotenvy::dotenv;
use tracing::{error, info};

use services::{DatabaseService, cleanup};
use utils::file_system;
use app::create_app;
use config::Config;
use logging::init_logging;
use services::establish_connection_pool;

#[actix_web::main] // Marks the main function as the Actix-web entry point
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load environment variables from .env file (for local development)
    if cfg!(debug_assertions) {
        dotenv().ok(); // Only load .env in development builds
    }

    // 2. Load and validate configuration
    let config = Config::from_env().map_err(|e| {
        eprintln!("Failed to load configuration: {:?}", e);
        "Failed to load configuration"
    })?;
    config.validate().map_err(|e| {
        eprintln!("Configuration validation failed: {:?}", e);
        "Configuration validation failed"
    })?;

    // 3. Initialize logging based on environment
    init_logging(&config).map_err(|e| {
        eprintln!("Failed to initialize logging: {:?}", e);
        "Failed to initialize logging"
    })?;
    
    info!(
        environment = ?config.environment,
        version = env!("CARGO_PKG_VERSION"),
        "WasmWiz starting up"
    );

    // 4. Database connection pool setup (skip in demo mode)
    let db_pool = if config.is_demo() {
        info!("Demo mode: Skipping database connection");
        None
    } else {
        info!("Attempting to connect to database...");
        let pool = establish_connection_pool(&config).await.map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            "Failed to connect to database"
        })?;
        info!("Database connection pool established");

        // 5. Run database migrations (essential for production)
        if !config.is_production() {
            info!("Running database migrations...");
            sqlx::migrate!("./migrations") // Path to your migrations directory
                .run(&pool)
                .await
                .map_err(|e| {
                    error!("Failed to run database migrations: {:?}", e);
                    "Failed to run database migrations"
                })?;
            info!("Database migrations complete");
        } else {
            info!("Production mode: Skipping automatic migrations");
        }
        
        Some(pool)
    };

    // 6. Start background cleanup tasks (skip database cleanup in demo mode)
    file_system::start_wasm_cleanup_task();
    if let Some(ref pool) = db_pool {
        cleanup::start_cleanup_tasks(DatabaseService::new(pool.clone()));
        info!("Background cleanup tasks started");
    } else {
        info!("Demo mode: Skipping database cleanup tasks");
    }

    // 7. Set up Actix-web server with production optimizations
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    let is_production = config.is_production();
    
    info!(
        host = %server_host,
        port = %server_port,
        "Starting Actix-web server"
    );

    let server = HttpServer::new(move || create_app(db_pool.clone(), config.clone()))
        .bind((server_host.as_str(), server_port))?;

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

    server.run().await?;

    info!("Server shut down gracefully");
    Ok(())
}
