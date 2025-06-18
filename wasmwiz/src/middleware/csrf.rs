// src/middleware/csrf.rs
use std::future::{Ready, ready};
use std::rc::Rc;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use sha2::{Sha256, Digest};
use rand::{Rng, distributions::Alphanumeric};
use tracing::warn;

pub struct CsrfMiddleware {
    secret: String,
}

impl CsrfMiddleware {
    #[allow(dead_code)]
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type Transform = CsrfService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfService {
            service: Rc::new(service),
            secret: self.secret.clone(),
        }))
    }
}

pub struct CsrfService<S> {
    service: Rc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for CsrfService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret = self.secret.clone();
        let service = self.service.clone();
        
        Box::pin(async move {
            // Skip CSRF for GET, HEAD, OPTIONS, and API endpoints (they use Bearer auth)
            let method = req.method();
            let path = req.path();
            
            if method == Method::GET || 
               method == Method::HEAD || 
               method == Method::OPTIONS ||
               path.starts_with("/api/") ||
               path.starts_with("/static/") ||
               path == "/health" {
                return service.call(req).await.map(|res| res.map_into_right_body());
            }
            
            // For POST requests to web forms, check CSRF token
            if method == Method::POST {
                // Get CSRF token from header or form data
                let csrf_token = req.headers()
                    .get("x-csrf-token")
                    .and_then(|h| h.to_str().ok())
                    .or_else(|| {
                        // For form submissions, we'd need to parse the body
                        // For now, we'll skip form parsing and rely on header
                        None
                    });
                
                if let Some(token) = csrf_token {
                    if !verify_csrf_token(&token, &secret) {
                        warn!("Invalid CSRF token from {}", req.connection_info().realip_remote_addr().unwrap_or("unknown"));
                        let response = HttpResponse::Forbidden()
                            .json(serde_json::json!({
                                "error": "Invalid CSRF token"
                            }));
                        return Ok(req.into_response(response).map_into_left_body());
                    }
                } else {
                    warn!("Missing CSRF token from {}", req.connection_info().realip_remote_addr().unwrap_or("unknown"));
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "CSRF token required"
                        }));
                    return Ok(req.into_response(response).map_into_left_body());
                }
            }
            
            service.call(req).await.map(|res| res.map_into_right_body())
        })
    }
}

/// Generate a CSRF token
pub fn generate_csrf_token(secret: &str) -> String {
    let random_part: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    
    let timestamp = chrono::Utc::now().timestamp();
    let data = format!("{}:{}", timestamp, random_part);
    
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    
    format!("{}:{}", data, hex::encode(hash))
}

/// Verify a CSRF token
fn verify_csrf_token(token: &str, secret: &str) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return false;
    }
    
    let timestamp: i64 = match parts[0].parse() {
        Ok(t) => t,
        Err(_) => return false,
    };
    
    // Token expires after 1 hour
    let now = chrono::Utc::now().timestamp();
    if now - timestamp > 3600 {
        return false;
    }
    
    let data = format!("{}:{}", parts[0], parts[1]);
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(data.as_bytes());
    let expected_hash = hex::encode(hasher.finalize());
    
    expected_hash == parts[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csrf_token_generation_and_verification() {
        let secret = "test_secret";
        let token = generate_csrf_token(secret);
        
        assert!(verify_csrf_token(&token, secret));
        assert!(!verify_csrf_token(&token, "wrong_secret"));
        assert!(!verify_csrf_token("invalid_token", secret));
    }
}
