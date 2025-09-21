//! # Wasm Wizard Core Library
//!
//! This is the core library crate for Wasm Wizard, a production-ready WebAssembly execution platform.
//! It provides all the essential components for building secure, scalable WASM execution services.
//!
//! ## Architecture Overview
//!
//! Wasm Wizard follows a modular architecture with clear separation of concerns:
//!
//! - **`config`**: Environment-based configuration management
//! - **`errors`**: Comprehensive error handling with HTTP status mapping
//! - **`models`**: Data structures for API payloads and database entities
//! - **`handlers`**: HTTP request handlers for all endpoints
//! - **`services`**: Business logic and external service integrations
//! - **`middleware`**: Authentication, rate limiting, and security middleware
//! - **`utils`**: Common utilities and helper functions
//! - **`app`**: Application setup and routing configuration
//!
//! ## Key Features
//!
//! - **Secure WASM Execution**: Sandboxed execution with resource limits
//! - **Authentication & Authorization**: API key-based authentication with tiers
//! - **Rate Limiting**: Distributed rate limiting with Redis support
//! - **Monitoring**: Comprehensive health checks and metrics
//! - **Error Handling**: Structured error responses with detailed messages
//! - **Configuration**: Environment-based configuration with validation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use wasm-wizard::{
//!     config::Config,
//!     app::create_app,
//!     services::establish_connection_pool,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::from_env()?;
//!     let db_pool = establish_connection_pool(&config).await?;
//!     let app = create_app(db_pool, config);
//!
//!     // Start the server...
//!     Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - All WASM execution is sandboxed with memory and time limits
//! - API keys are hashed using secure cryptographic functions
//! - Input validation prevents injection attacks
//! - Rate limiting prevents abuse and DoS attacks
//! - Comprehensive audit logging for compliance

pub mod app;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod logging;
pub mod middleware;
pub mod models;
pub mod monitoring;
pub mod services;
pub mod utils;
pub mod wasm;

pub use app::{create_app, AppState};
pub use config::{Config, Environment};
pub use logging::init_logging;
pub use services::database::establish_connection_pool;
