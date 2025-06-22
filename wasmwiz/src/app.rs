// src/app.rs
use crate::config::Config;
use crate::handlers::{api_keys, execute, health, web as web_handlers};
use crate::middleware::{InputValidationMiddleware, RateLimitMiddleware, SecurityHeadersMiddleware};
use crate::middleware::pre_auth::PreAuth;
use crate::services::{DatabaseService, RedisService};
use actix_files as fs;
use actix_web::{App, web};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,  // Always required now
    pub db_service: DatabaseService,  // Always required now
    pub config: Config,
    #[allow(dead_code)] // Reserved for future Redis integration
    pub redis_service: Option<RedisService>,
}

pub fn create_app(
    db_pool: PgPool,  // No longer optional
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
    
    // Initialize Redis service if URL is available
    let redis_service = match RedisService::new(&config.redis_url) {
        Ok(service) => {
            tracing::info!("Redis service initialized successfully");
            Some(service)
        },
        Err(e) => {
            if config.is_production() {
                panic!("FATAL: Redis is required in production but could not be initialized: {}", e);
            }
            tracing::warn!("Failed to initialize Redis service, falling back to in-memory rate limiting: {}", e);
            None
        }
    };
    
    // Create rate limit middleware with Redis if available
    let rate_limit_middleware = if let Some(redis) = redis_service.clone() {
        tracing::info!("Using Redis-based rate limiting");
        RateLimitMiddleware::with_redis(redis)
    } else {
        tracing::warn!("Using in-memory rate limiting");
        RateLimitMiddleware::new()
    };
    
    let security_middleware = SecurityHeadersMiddleware::new();
    let input_validation_middleware = InputValidationMiddleware::new();

    let mut app = App::new()
        .app_data(web::Data::new(AppState {
            db_pool: db_pool.clone(),
            db_service: db_service.clone(),
            config: config.clone(),
            redis_service: redis_service.clone(),
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
        .service(fs::Files::new("/static", "./static").show_files_listing());
    
    // Add API routes conditionally
    if config.auth_required {
        app = app.service(
            web::scope("/api")
                .wrap(PreAuth::new(db_service.clone()))
                .wrap(RateLimitMiddleware::with_redis(
                    redis_service.clone().unwrap_or_else(|| {
                        RedisService::new(&config.redis_url).unwrap()
                    })
                ))
                .service(web::resource("/execute").post(execute::execute_wasm))
        );
    } else {
        app = app.service(
            web::scope("/api")
                .service(web::resource("/execute").post(execute::execute_wasm_no_auth))
                .service(
                    web::resource("/debug-execute")
                        .route(web::post().to(execute::debug_execute))
                )
        );
    }

    app.service(
            web::scope("/admin")
                .service(web::resource("/api-keys").post(api_keys::create_api_key))
                .service(web::resource("/api-keys/{email}").get(api_keys::list_api_keys))
                .service(
                    web::resource("/api-keys/{id}/deactivate").post(api_keys::deactivate_api_key),
                ),
        )
}
