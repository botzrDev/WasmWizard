mod models;
mod utils;
mod errors;
mod handlers;
mod middleware;
mod services;
mod config;
mod app;

use actix_web::HttpServer;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use dotenvy::dotenv;

use utils::file_system;
use services::{DatabaseService, cleanup, establish_connection_pool};
use config::Config;
use app::create_app;

#[actix_web::main] // Marks the main function as the Actix-web entry point
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize logging
    // Filters logs based on RUST_LOG environment variable (e.g., RUST_LOG=info,wasmwiz=debug)
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Logger initialized.");

    // 2. Load environment variables from .env file (for local development)
    dotenv().ok(); // Doesn't fail if .env is not found
    info!(".env file loaded (if present).");

    // 3. Load and validate configuration
    let config = Config::from_env()
        .map_err(|e| {
            error!("Failed to load configuration: {:?}", e);
            "Failed to load configuration"
        })?;
    config.validate()
        .map_err(|e| {
            error!("Configuration validation failed: {:?}", e);
            "Configuration validation failed"
        })?;
    info!("Configuration loaded and validated.");

    // 4. Database connection pool setup
    info!("Attempting to connect to database...");
    let db_pool = establish_connection_pool(&config).await
        .map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            "Failed to connect to database"
        })?;
    info!("Database connection pool established.");

    // 5. Run database migrations (optional for prod, but good for dev/CI)
    info!("Running database migrations...");
    sqlx::migrate!("./migrations") // Path to your migrations directory
        .run(&db_pool)
        .await
        .map_err(|e| {
            error!("Failed to run database migrations: {:?}", e);
            "Failed to run database migrations"
        })?;
    info!("Database migrations complete.");

    // 6. Create database service (no longer needed here, moved to create_app)
    // let db_service = DatabaseService::new(db_pool.clone());
    // info!("Database service initialized.");

    // 7. Start background cleanup tasks
    file_system::start_wasm_cleanup_task();
    // Pass db_service to cleanup tasks if needed, or refactor cleanup to take db_pool
    // For now, assuming cleanup can work with just db_pool or is self-contained
    cleanup::start_cleanup_tasks(DatabaseService::new(db_pool.clone())); // Re-initialize for cleanup if needed
    info!("Background cleanup tasks started.");

    // 8. Set up Actix-web server
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    info!("Starting Actix-web server on {}:{}", server_host, server_port);
    HttpServer::new(move || {
        create_app(db_pool.clone(), config.clone())
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await?;

    info!("Server shut down gracefully.");
    Ok(())
}
