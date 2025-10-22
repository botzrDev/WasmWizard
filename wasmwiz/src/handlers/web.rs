//! # Web Interface Handlers
//!
//! This module provides the web interface handlers for the Wasm Wizard application.
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
//! use wasm-wizard::handlers::web;
//!
//! let app = App::new()
//!     .route("/", web::get().to(web::index))
//!     .route("/api-keys", web::get().to(web::api_keys));
//! ```

use crate::app::AppState;
use crate::errors::ApiError;
use crate::handlers::api_keys;
use crate::middleware::generate_csrf_token;
use crate::middleware::pre_auth::AuthContext;
use actix_web::{web, HttpResponse, Result as ActixResult};
use askama_actix::{Template, TemplateToResponse};

/// Helper function to get advertisement data from AppState
fn get_ad_data(app_state: &AppState) -> (bool, String, String, String) {
    use crate::models::AdPlacement;
    
    let adsense_enabled = app_state.config.ads_enabled && app_state.config.adsense_client_id.is_some();
    let adsense_client_id = app_state.config.adsense_client_id.clone().unwrap_or_default();
    
    let (header_ad, footer_ad) = if app_state.config.ads_enabled {
        (
            app_state.ad_manager.render_placement(AdPlacement::Header),
            app_state.ad_manager.render_placement(AdPlacement::Footer),
        )
    } else {
        (String::new(), String::new())
    };
    
    (adsense_enabled, adsense_client_id, header_ad, footer_ad)
}

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
    /// AdSense enabled flag
    pub adsense_enabled: bool,
    /// AdSense client ID
    pub adsense_client_id: String,
    /// Header advertisement HTML
    pub header_ad: String,
    /// Footer advertisement HTML
    pub footer_ad: String,
}

/// Template for the API keys management page.
///
/// Renders the api_keys.html template with CSRF protection and navigation state.
#[derive(Template)]
#[template(path = "api_keys.html")]
pub struct ApiKeysTemplate {
    pub title: String,
    pub csrf_token: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the documentation page.
#[derive(Template)]
#[template(path = "docs.html")]
pub struct DocsTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the examples page.
#[derive(Template)]
#[template(path = "examples.html")]
pub struct ExamplesTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the pricing page.
#[derive(Template)]
#[template(path = "pricing.html")]
pub struct PricingTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the FAQ page.
#[derive(Template)]
#[template(path = "faq.html")]
pub struct FaqTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the support page.
#[derive(Template)]
#[template(path = "support.html")]
pub struct SupportTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the security page.
#[derive(Template)]
#[template(path = "security.html")]
pub struct SecurityTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the terms of service page.
#[derive(Template)]
#[template(path = "terms.html")]
pub struct TermsTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
}

/// Template for the privacy policy page.
#[derive(Template)]
#[template(path = "privacy.html")]
pub struct PrivacyTemplate {
    pub title: String,
    pub active_page: &'static str,
    pub adsense_enabled: bool,
    pub adsense_client_id: String,
    pub header_ad: String,
    pub footer_ad: String,
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
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = IndexTemplate {
        title: "Execute WebAssembly".to_string(),
        csrf_token,
        active_page: "index",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
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
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = ApiKeysTemplate {
        title: "API Key Management".to_string(),
        csrf_token,
        active_page: "api-keys",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the documentation page.
pub async fn docs(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = DocsTemplate {
        title: "API Documentation".to_string(),
        active_page: "docs",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the examples page.
pub async fn examples(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = ExamplesTemplate {
        title: "WebAssembly Examples".to_string(),
        active_page: "examples",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the pricing page.
pub async fn pricing(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = PricingTemplate {
        title: "Pricing Plans".to_string(),
        active_page: "pricing",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the FAQ page.
pub async fn faq(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = FaqTemplate {
        title: "Frequently Asked Questions".to_string(),
        active_page: "faq",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the support page.
pub async fn support(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = SupportTemplate {
        title: "Get Support".to_string(),
        active_page: "support",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the security page.
pub async fn security(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = SecurityTemplate {
        title: "Security & Compliance".to_string(),
        active_page: "security",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the terms of service page.
pub async fn terms(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = TermsTemplate {
        title: "Terms of Service".to_string(),
        active_page: "terms",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
    };
    Ok(template.to_response())
}

/// Serve the privacy policy page.
pub async fn privacy(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let (adsense_enabled, adsense_client_id, header_ad, footer_ad) = get_ad_data(&app_state);
    
    let template = PrivacyTemplate {
        title: "Privacy Policy".to_string(),
        active_page: "privacy",
        adsense_enabled,
        adsense_client_id,
        header_ad,
        footer_ad,
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
    auth_context: AuthContext,
    form: web::Form<api_keys::CreateApiKeyRequest>,
) -> ActixResult<HttpResponse, ApiError> {
    // Ensure the target email is derived from the authenticated caller
    let mut payload = form.into_inner();
    payload.user_email = auth_context.user.email.clone();

    // Use the existing create_api_key function with authenticated context
    let json_req = web::Json(payload);
    api_keys::create_api_key(app_state, auth_context, json_req).await
}

/// Generate CSRF token endpoint
pub async fn csrf_token(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let token = generate_csrf_token(&app_state.config.api_salt);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "csrf_token": token
    })))
}
