//! # Business Logic Services
//!
//! This module contains the core business logic services for Wasm Wizard.
//! These services handle data persistence, caching, and background processing.
//!
//! ## Services Overview
//!
//! - **`database`**: PostgreSQL connection management and data access
//! - **`redis`**: Redis client for caching and distributed rate limiting
//! - **`cleanup`**: Background tasks for cleaning up temporary files and data
//!
//! ## Architecture
//!
//! Services follow a clean architecture pattern:
//!
//! - **Repository Pattern**: Data access is abstracted through service interfaces
//! - **Dependency Injection**: Services are injected into handlers via Actix-web's data system
//! - **Async/Await**: All operations are asynchronous for scalability
//! - **Error Handling**: Comprehensive error handling with proper error types
//!
//! ## Usage
//!
//! ```rust,no_run
//! use wasm-wizard::services::{DatabaseService, RedisService};
//!
//! // In application setup
//! let db_service = DatabaseService::new(db_pool);
//! let redis_service = RedisService::new(&config.redis_url)?;
//!
//! // In handlers
//! async fn handler(db: web::Data<DatabaseService>) -> Result<HttpResponse> {
//!     let user = db.get_user(user_id).await?;
//!     Ok(HttpResponse::Ok().json(user))
//! }
//! ```

pub mod cleanup;
pub mod database;
pub mod redis;

pub use database::{establish_connection_pool, DatabaseService};
pub use redis::RedisService;
