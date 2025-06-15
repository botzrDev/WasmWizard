// src/handlers/web.rs
use actix_web::{web, HttpResponse, Result as ActixResult};
use askama::Template;
use crate::errors::ApiError;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
}

/// Serve the main upload interface
pub async fn index() -> ActixResult<HttpResponse, ApiError> {
    let template = IndexTemplate {
        title: "Execute WebAssembly".to_string(),
    };
    
    let html = template.render().map_err(|e| {
        tracing::error!("Template rendering failed: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Template rendering failed"))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[derive(Template)]
#[template(path = "api_keys.html")]
struct ApiKeysTemplate {
    title: String,
}

/// Serve the API keys management page
pub async fn api_keys() -> ActixResult<HttpResponse, ApiError> {
    let template = ApiKeysTemplate {
        title: "API Key Management".to_string(),
    };
    
    let html = template.render().map_err(|e| {
        tracing::error!("Template rendering failed: {}", e);
        ApiError::InternalError(anyhow::anyhow!("Template rendering failed"))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
