//! # HTTP Middleware
//!
//! This module provides HTTP middleware components for request processing,
//! authentication, security, and rate limiting.
//!
//! ## Middleware Components
//!
//! ### Authentication & Authorization
//! - **`auth`**: API key authentication and authorization
//! - **`pre_auth`**: Pre-authentication context extraction
//!
//! ### Security
//! - **`security`**: Security headers (CSP, HSTS, etc.)
//! - **`csrf`**: CSRF protection for web forms
//! - **`input_validation`**: Request payload validation
//!
//! ### Rate Limiting
//! - **`rate_limit`**: In-memory rate limiting
//! - **`redis_rate_limit`**: Redis-backed distributed rate limiting
//! - **`distributed_rate_limit`**: Unified rate limiting interface
//! - **`rate_limit_middleware`**: Actix-web middleware integration
//!
//! ## Security Architecture
//!
//! The middleware stack provides defense in depth:
//!
//! 1. **Input Validation**: Validate and sanitize all input
//! 2. **Authentication**: Verify API key validity
//! 3. **Authorization**: Check user permissions and tiers
//! 4. **Rate Limiting**: Prevent abuse and DoS attacks
//! 5. **Security Headers**: Add security headers to responses
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use actix_web::{web, App};
//! use wasm-wizard::middleware::*;
//!
//! let app = App::new()
//!     .wrap(SecurityHeadersMiddleware)
//!     .wrap(RateLimitMiddleware::new(...))
//!     .wrap(AuthMiddleware)
//!     .service(web::scope("/api").service(my_handler));
//! ```
//!
//! ## Configuration
//!
//! Middleware behavior is controlled through the application configuration:
//!
//! - Rate limits vary by subscription tier
//! - Authentication can be disabled in development
//! - Security headers are configurable
//! - CSRF protection is environment-aware

pub mod admin_auth;
pub mod auth;
pub mod csrf;
pub mod distributed_rate_limit;
pub mod input_validation;
pub mod master_admin;
pub mod pre_auth;
pub mod rate_limit;
pub mod rate_limit_middleware;
pub mod redis_rate_limit;
pub mod security;
pub mod tier_access;

pub use admin_auth::AdminAuthMiddleware;
pub use auth::AuthContext;
pub use csrf::generate_csrf_token;
pub use input_validation::InputValidationMiddleware;
pub use master_admin::{AdminRole, MasterAdminMiddleware};
pub use security::SecurityHeadersMiddleware;
pub use tier_access::{RequiredTier, TierAccessMiddleware};
