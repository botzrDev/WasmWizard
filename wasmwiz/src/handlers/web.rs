// src/handlers/web.rs
use crate::app::AppState;
use crate::errors::ApiError;
use crate::handlers::api_keys;
use crate::middleware::generate_csrf_token;
use actix_web::{web, HttpResponse, Result as ActixResult};
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub csrf_token: String,
    pub active_page: &'static str,
}

#[derive(Template)]
#[template(path = "api_keys.html")]
pub struct ApiKeysTemplate {
    pub title: String,
    pub csrf_token: String,
    pub active_page: &'static str,
}

/// Serve the main upload interface
pub async fn index(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let csrf_token = generate_csrf_token(&app_state.config.api_salt);
    let template = IndexTemplate {
        title: "Execute WebAssembly".to_string(),
        csrf_token,
        active_page: "index",
    };
    Ok(template.to_response())
}

/// Serve the API keys management page
pub async fn api_keys(app_state: web::Data<AppState>) -> ActixResult<HttpResponse, ApiError> {
    let csrf_token = generate_csrf_token(&app_state.config.api_salt);
    let template = ApiKeysTemplate {
        title: "API Key Management".to_string(),
        csrf_token,
        active_page: "api-keys",
    };
    Ok(template.to_response())
}

/// Handle web form upload (placeholder - directs to AJAX)
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
