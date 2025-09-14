//! # Web Interface Handlers
//!
//! This module provides the web interface handlers for the WasmWiz application.
//! It serves HTML pages for the user interface and handles web-based interactions.
//!
//! ## Pages Served
//!
//! - **Home Page** (`/`): Main WASM execution interface
//! - **API Keys Page** (`/api-keys`): API key management interface
//! - **Upload Interface**: WASM module upload forms
//!
//! ## Templates
//!
//! Uses Askama templates for server-side rendering:
//!
//! - `index.html`: Main application page with upload form
//! - `api_keys.html`: API key management interface
//!
//! ## Security Features
//!
//! - **CSRF Protection**: Tokens generated for all forms
//! - **Authentication**: Required for sensitive operations
//! - **Input Validation**: Client and server-side validation
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use actix_web::{web, App};
//! use wasmwiz::handlers::web;
//!
//! let app = App::new()
//!     .route("/", web::get().to(web::index))
//!     .route("/api-keys", web::get().to(web::api_keys));
//! ```

use crate::app::AppState;
use crate::errors::ApiError;
use crate::handlers::api_keys;
use crate::middleware::generate_csrf_token;
use actix_web::{web, HttpResponse, Result as ActixResult};
use askama_actix::{Template, TemplateToResponse};

/// Template for the main application page.
///
/// Renders the index.html template with CSRF protection and navigation state.
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Page title for browser tab
    pub title: String,
    /// CSRF token for form protection
    pub csrf_token: String,
    /// Active page identifier for navigation highlighting
    pub active_page: &'static str,
}

/// Template for the API keys management page.
///
/// Renders the api_keys.html template with CSRF protection and navigation state.
#[derive(Template)]
#[template(path = "api_keys.html")]
pub struct ApiKeysTemplate {
    /// Page title for browser tab
    pub title: String,
    /// CSRF token for form protection
    pub csrf_token: String,
    /// Active page identifier for navigation highlighting
    pub active_page: &'static str,
}

/// Serve the main WASM execution interface.
///
/// This is the home page of the application, providing the primary interface
/// for users to upload and execute WebAssembly modules.
///
/// # Returns
///
/// Returns the rendered HTML page with:
/// - WASM upload form
/// - CSRF protection token
/// - Navigation and user interface elements
///
/// # Security
///
/// - Requires user authentication
/// - Includes CSRF protection
/// - Validates user permissions
pub async fn index(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let csrf_token = generate_csrf_token(&app_state.config.api_salt);
    let template = IndexTemplate {
        title: "Execute WebAssembly".to_string(),
        csrf_token,
        active_page: "index",
    };
    Ok(template.to_response())
}

/// Serve the API keys management page.
///
/// Provides an interface for users to view, create, and manage their API keys.
/// This page allows users to generate new keys and revoke existing ones.
///
/// # Returns
///
/// Returns the rendered HTML page with:
/// - List of user's API keys
/// - Key generation form
/// - Key revocation options
/// - CSRF protection
///
/// # Security
///
/// - Requires user authentication
/// - Shows only user's own keys
/// - Includes CSRF protection for forms
pub async fn api_keys(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let csrf_token = generate_csrf_token(&app_state.config.api_salt);
    let template = ApiKeysTemplate {
        title: "API Key Management".to_string(),
        csrf_token,
        active_page: "api-keys",
    };
    Ok(template.to_response())
}

/// Handle web form upload (placeholder - directs to AJAX).
///
/// This endpoint provides a fallback for users who prefer traditional form
/// submission over AJAX. It directs users to use the AJAX endpoint for
/// better user experience.
///
/// # Returns
///
/// Returns instructions to use the AJAX upload endpoint instead.
///
/// # Future Enhancement
///
/// This could be enhanced to handle direct form submissions if needed
/// for compatibility with older browsers or specific use cases.
pub async fn upload_form() -> ActixResult<HttpResponse, ApiError> {
    // For now, return a simple message directing users to use AJAX
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("<h1>Use the Execute Button</h1><p>Please use the Execute button on the main page for file uploads.</p><a href='/'>Go Back</a>"))
}

/// Handle web form API key generation
pub async fn generate_key_form(
    app_state: web::Data<AppState>,
    form: web::Form<api_keys::CreateApiKeyRequest>,
) -> ActixResult<HttpResponse, ApiError> {
    // Use the existing create_api_key function
    let json_req = web::Json(form.into_inner());
    api_keys::create_api_key(app_state, json_req).await
}

/// Generate CSRF token endpoint
pub async fn csrf_token(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let token = generate_csrf_token(&app_state.config.api_salt);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "csrf_token": token
    })))
}
