mod models;
mod utils;
mod errors;
mod handlers;
mod middleware;
mod services;
mod config;

use actix_web::{web, App, HttpServer};
use actix_files as fs;
use sqlx::PgPool;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use dotenvy::dotenv;

use utils::file_system;
use handlers::{health, execute, web as web_handlers, api_keys};
use middleware::{AuthMiddleware, RateLimitMiddleware, SecurityHeadersMiddleware, InputValidationMiddleware};
use services::{DatabaseService, cleanup};
use config::Config;

pub struct AppState {
    pub db_pool: PgPool,
    pub db_service: DatabaseService,
    pub config: Config,
}

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
    let database_url = config.database_url.clone();
    info!("Attempting to connect to database...");
    let db_pool = PgPool::connect(&database_url).await
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

    // 6. Create database service
    let db_service = DatabaseService::new(db_pool.clone());
    info!("Database service initialized.");

    // 7. Start background cleanup tasks
    file_system::start_wasm_cleanup_task();
    cleanup::start_cleanup_tasks(db_service.clone());
    info!("Background cleanup tasks started.");

    // 8. Set up Actix-web server
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    info!("Starting Actix-web server on {}:{}", server_host, server_port);
    HttpServer::new(move || {
        let auth_middleware = AuthMiddleware::new(db_service.clone());
        let rate_limit_middleware = RateLimitMiddleware::new();
        let security_middleware = SecurityHeadersMiddleware::new();
        let input_validation_middleware = InputValidationMiddleware::new();
        
        App::new()
            .app_data(web::Data::new(AppState { 
                db_pool: db_pool.clone(),
                db_service: db_service.clone(),
                config: config.clone(),
            }))
            .wrap(security_middleware)
            .wrap(input_validation_middleware)
            // Health check endpoint (no auth required)
            .service(web::resource("/health").get(health::health_check))
            // Web interface routes (no auth required)
            .service(web::resource("/").get(web_handlers::index))
            .service(web::resource("/api-keys").get(web_handlers::api_keys))
            .service(web::resource("/upload").post(web_handlers::upload_form))
            .service(web::resource("/generate-key").post(web_handlers::generate_key_form))
            .service(web::resource("/csrf-token").get(web_handlers::csrf_token))
            // Static file serving (no auth required)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Protected API endpoints with auth and rate limiting
            .service(
                web::scope("/api")
                    .wrap(rate_limit_middleware)
                    .wrap(auth_middleware)
                    .service(web::resource("/execute").post(execute::execute_wasm))
            )
            // API key management endpoints (no auth required for now - would need admin auth in production)
            .service(
                web::scope("/admin")
                    .service(web::resource("/api-keys").post(api_keys::create_api_key))
                    .service(web::resource("/api-keys/{email}").get(api_keys::list_api_keys))
                    .service(web::resource("/api-keys/{id}/deactivate").post(api_keys::deactivate_api_key))
            )
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await?;

    info!("Server shut down gracefully.");
    Ok(())
}
