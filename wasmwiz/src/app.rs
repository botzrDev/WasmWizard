// src/app.rs
use crate::config::Config;
use crate::handlers::{api_keys, execute, health, web as web_handlers};
use crate::middleware::{
    AuthMiddleware, InputValidationMiddleware, RateLimitMiddleware, SecurityHeadersMiddleware,
};
use crate::services::DatabaseService;
use actix_files as fs;
use actix_web::{App, web};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub db_service: DatabaseService,
    pub config: Config,
}

pub fn create_app(
    db_pool: PgPool,
    config: Config,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let db_service = DatabaseService::new(db_pool.clone());
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
        // Health check endpoints (Kubernetes probes)
        .service(web::resource("/healthz").get(health::liveness_probe))
        .service(web::resource("/readyz").get(health::readiness_probe))
        // Prometheus metrics endpoint
        .service(web::resource("/metrics").get(health::prometheus_metrics))
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
                .service(web::resource("/execute").post(execute::execute_wasm)),
        )
        // API key management endpoints (no auth required for now - would need admin auth in production)
        .service(
            web::scope("/admin")
                .service(web::resource("/api-keys").post(api_keys::create_api_key))
                .service(web::resource("/api-keys/{email}").get(api_keys::list_api_keys))
                .service(
                    web::resource("/api-keys/{id}/deactivate").post(api_keys::deactivate_api_key),
                ),
        )
}
