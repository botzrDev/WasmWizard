mod models;
mod utils;
mod errors;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::PgPool;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use dotenvy::dotenv;

use errors::ApiError;
use utils::file_system;

pub struct AppState {
    pub db_pool: PgPool,
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

    // 3. Database connection pool setup
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set in .env or environment")?;
    info!("Attempting to connect to database...");
    let db_pool = PgPool::connect(&database_url).await
        .map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            "Failed to connect to database"
        })?;
    info!("Database connection pool established.");

    // 4. Run database migrations (optional for prod, but good for dev/CI)
    info!("Running database migrations...");
    sqlx::migrate!("./migrations") // Path to your migrations directory
        .run(&db_pool)
        .await
        .map_err(|e| {
            error!("Failed to run database migrations: {:?}", e);
            "Failed to run database migrations"
        })?;
    info!("Database migrations complete.");

    // 5. Start Wasm cleanup task in a background thread
    file_system::start_wasm_cleanup_task();
    info!("Wasm temporary file cleanup task started.");

    // 6. Set up Actix-web server
    info!("Starting Actix-web server...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db_pool: db_pool.clone() })) // Pass shared state to handlers
            // TODO: Add middleware here (e.g., tracing, authentication, rate limiting)
            // .wrap(TracingLogger::default()) // Example: enable request logging
            // TODO: Define API routes here
            .service(web::resource("/execute").post(todo!())) // Placeholder for your /execute endpoint
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    info!("Server shut down gracefully.");
    Ok(())
}
