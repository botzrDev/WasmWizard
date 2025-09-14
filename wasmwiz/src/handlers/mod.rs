//! # HTTP Request Handlers
//!
//! This module contains all HTTP request handlers for the WasmWiz API and web interface.
//! Handlers are organized by functionality and follow RESTful conventions.
//!
//! ## Handler Categories
//!
//! ### API Endpoints
//! - **`execute`**: Core WASM execution functionality
//! - **`api_keys`**: API key management operations
//! - **`health`**: Health check and monitoring endpoints
//!
//! ### Web Interface
//! - **`web`**: HTML page serving and web form handling
//!
//! ## Handler Architecture
//!
//! All handlers follow consistent patterns:
//!
//! - **Async Functions**: All handlers are `async fn` for scalability
//! - **Error Handling**: Return `Result<T, ApiError>` with proper error types
//! - **Authentication**: Use middleware for auth, extract context from requests
//! - **Validation**: Input validation using middleware and manual checks
//! - **Logging**: Structured logging with appropriate log levels
//!
//! ## Common Patterns
//!
//! ```rust,no_run
//! use actix_web::{web, HttpResponse, Result};
//! use wasmwiz::{errors::ApiError, middleware::pre_auth::AuthContext};
//!
//! pub async fn my_handler(
//!     auth_context: AuthContext,           // Authentication context
//!     app_state: web::Data<AppState>,      // Application state
//!     path_params: web::Path<String>,      // URL path parameters
//!     query_params: web::Query<MyQuery>,   // Query parameters
//!     json_body: web::Json<MyRequest>,     // JSON request body
//! ) -> Result<HttpResponse, ApiError> {
//!     // Handler logic here
//!     Ok(HttpResponse::Ok().json(response))
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - **Input Validation**: All inputs are validated before processing
//! - **Rate Limiting**: Applied at handler level via middleware
//! - **Authentication**: Required for sensitive operations
//! - **Authorization**: Permission checks based on user roles/tiers
//! - **Audit Logging**: All operations are logged for compliance

pub mod api_keys;
pub mod execute;
pub mod health;
pub mod web;
