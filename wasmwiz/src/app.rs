// src/app.rs
use crate::config::Config;
use crate::handlers::{admin, api_keys, execute, health, wasm_modules, web as web_handlers};
use crate::middleware::distributed_rate_limit::{create_rate_limiter, RateLimitService};
use crate::middleware::pre_auth::PreAuth;

use crate::middleware::rate_limit_middleware::RateLimitMiddleware;

use crate::middleware::{
    InputValidationMiddleware, MasterAdminMiddleware, RequiredTier, SecurityHeadersMiddleware,
    TierAccessMiddleware,
};
use crate::services::{DatabaseService, RedisService};
use actix_files as fs;
use actix_web::{web, App};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,             // Always required now
    pub db_service: DatabaseService, // Always required now
    pub config: Arc<Config>,
    #[allow(dead_code)] // Reserved for future Redis integration
    pub redis_service: Option<RedisService>,
}

pub fn create_app(
    db_pool: PgPool, // No longer optional
    config: Arc<Config>,
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
    let redis_service = if config.redis_enabled {
        match RedisService::new(&config.redis_url) {
            Ok(service) => {
                tracing::info!("Redis service initialized successfully");
                Some(service)
            }
            Err(error) => {
                if config.is_production() {
                    panic!(
                        "FATAL: Redis is required in production but could not be initialized: {}",
                        error
                    );
                }
                tracing::warn!(
                    "Failed to initialize Redis service, falling back to in-memory rate limiting: {}",
                    error
                );
                None
            }
        }
    } else {
        tracing::info!("Redis integration disabled via configuration");
        None
    };

    // Create rate limit middleware with Redis if available
    let shared_limiter = if redis_service.is_some() {
        tracing::info!("Using Redis-based rate limiting");
        create_rate_limiter(Some(config.redis_url.as_str()))
    } else {
        tracing::warn!("Using in-memory rate limiting");
        create_rate_limiter(None)
    };
    let rate_limit_service = RateLimitService::new(shared_limiter);

    let security_middleware = SecurityHeadersMiddleware::new(Arc::clone(&config));
    let input_validation_middleware = InputValidationMiddleware::new();
    let rate_limit_data = web::Data::new(rate_limit_service.clone());

    let mut app = App::new()
        .app_data(web::Data::new(AppState {
            db_pool: db_pool.clone(),
            db_service: db_service.clone(),
            config: Arc::clone(&config),
            redis_service: redis_service.clone(),
        }))
        .app_data(rate_limit_data.clone())
        .wrap(security_middleware)
        .wrap(input_validation_middleware)
        // .wrap(DistributedRateLimitMiddleware::new()) // Temporarily disabled
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
        .service(web::resource("/docs").get(web_handlers::docs))
        .service(web::resource("/examples").get(web_handlers::examples))
        .service(web::resource("/pricing").get(web_handlers::pricing))
        .service(web::resource("/faq").get(web_handlers::faq))
        .service(web::resource("/support").get(web_handlers::support))
        .service(web::resource("/security").get(web_handlers::security))
        .service(web::resource("/terms").get(web_handlers::terms))
        .service(web::resource("/privacy").get(web_handlers::privacy))
        .service(web::resource("/upload").post(web_handlers::upload_form))
        .service(
            web::resource("/generate-key")
                .wrap(PreAuth::new(db_service.clone()))
                .post(web_handlers::generate_key_form),
        )
        .service(web::resource("/csrf-token").get(web_handlers::csrf_token))
        // Static file serving (no auth required)
        .service(fs::Files::new("/static", "./static"));

    // Add API routes with proper authentication and tier-based access
    if config.auth_required {
        app = app
            .service(
                web::scope("/api")
                    .wrap(RateLimitMiddleware::new())
                    .wrap(PreAuth::new(db_service.clone()))
                    .app_data(rate_limit_data.clone())
                    // Public API endpoints (require auth but any tier)
                    .service(
                        web::resource("/execute")
                            .post(execute::execute_wasm)
                            .wrap(TierAccessMiddleware::new(RequiredTier::Free)),
                    )
                    // WASM module management endpoints (Free tier and up)
                    .service(
                        web::scope("/wasm")
                            .wrap(TierAccessMiddleware::new(RequiredTier::Free))
                            .service(web::resource("/upload").post(wasm_modules::upload_module))
                            .service(web::resource("/modules").get(wasm_modules::list_modules))
                            .service(
                                web::resource("/modules/{id}").delete(wasm_modules::delete_module),
                            ),
                    )
                    // API key management endpoints (Free tier and up)
                    .service(
                        web::scope("/auth")
                            .wrap(TierAccessMiddleware::new(RequiredTier::Free))
                            .service(web::resource("/keys").post(api_keys::create_api_key))
                            .service(web::resource("/keys").get(api_keys::list_user_api_keys))
                            .service(
                                web::resource("/keys/{id}").delete(api_keys::deactivate_api_key),
                            ), // TODO: Implement JWT refresh endpoint
                               //.service(web::resource("/refresh").post(api_keys::refresh_token))
                    )
                    // Basic tier endpoints
                    .service(
                        web::scope("/modules").wrap(TierAccessMiddleware::new(RequiredTier::Basic)), // Extended module features would go here
                    )
                    // Pro tier endpoints
                    .service(
                        web::scope("/analytics").wrap(TierAccessMiddleware::new(RequiredTier::Pro)), // Analytics endpoints would go here
                    )
                    // Enterprise tier endpoints
                    .service(
                        web::scope("/enterprise")
                            .wrap(TierAccessMiddleware::new(RequiredTier::Enterprise)), // Enterprise features would go here
                    ),
            )
            // Admin portal (master admin only)
            .service(
                web::scope("/admin")
                    .wrap(RateLimitMiddleware::new())
                    .wrap(MasterAdminMiddleware::support_admin_or_above())
                    .wrap(PreAuth::new(db_service.clone()))
                    // Dashboard
                    .service(web::resource("").get(admin::admin_dashboard))
                    .service(web::resource("/").get(admin::admin_dashboard))
                    // User Management (System Admin or above)
                    .service(
                        web::scope("/users")
                            .wrap(MasterAdminMiddleware::system_admin_or_above())
                            .service(web::resource("").get(admin::admin_users))
                            .service(web::resource("/").get(admin::admin_users))
                            .service(web::resource("/create").post(admin::create_user))
                            .service(web::resource("/{user_id}/tier").put(admin::update_user_tier)),
                    )
                    // API Key Management (System Admin or above)
                    .service(
                        web::scope("/api-keys")
                            .wrap(MasterAdminMiddleware::system_admin_or_above())
                            .service(web::resource("").get(admin::admin_api_keys))
                            .service(web::resource("/").get(admin::admin_api_keys))
                            .service(web::resource("/create").post(api_keys::create_api_key))
                            .service(web::resource("/{email}").get(api_keys::list_api_keys))
                            .service(
                                web::resource("/{id}/deactivate")
                                    .post(api_keys::deactivate_api_key),
                            ),
                    )
                    // Analytics (Support Admin or above)
                    .service(web::resource("/analytics").get(admin::admin_analytics))
                    // Tier Management (Master Admin only)
                    .service(
                        web::scope("/tiers")
                            .wrap(MasterAdminMiddleware::master_only())
                            .service(web::resource("").get(admin::admin_tiers))
                            .service(web::resource("/").get(admin::admin_tiers))
                            .service(web::resource("/create").post(admin::create_tier)),
                    )
                    // System Control (Master Admin only)
                    .service(
                        web::scope("/system")
                            .wrap(MasterAdminMiddleware::master_only())
                            .service(web::resource("/status").get(admin::system_status))
                            .service(
                                web::resource("/emergency-shutdown")
                                    .post(admin::emergency_shutdown),
                            ),
                    ),
            );
    } else {
        // Development mode - limited auth requirements but still secure admin
        tracing::warn!("Running in development mode with reduced authentication requirements");
        app = app
            .service(
                web::scope("/api")
                    .wrap(RateLimitMiddleware::new())
                    .app_data(rate_limit_data.clone())
                    .service(web::resource("/execute").post(execute::execute_wasm_no_auth)),
            )
            .service(
                web::scope("/admin")
                    .wrap(RateLimitMiddleware::new())
                    .wrap(MasterAdminMiddleware::support_admin_or_above())
                    .wrap(PreAuth::new(db_service.clone()))
                    .service(web::resource("/api-keys").post(api_keys::create_api_key))
                    .service(web::resource("/api-keys/{email}").get(api_keys::list_api_keys))
                    .service(
                        web::resource("/api-keys/{id}/deactivate")
                            .post(api_keys::deactivate_api_key),
                    ),
            );
    }

    app
}
