// src/middleware/auth.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, Result, HttpMessage,
    http::header::{HeaderValue, AUTHORIZATION},
};
use futures_util::future::{ready, Ready, LocalBoxFuture};
use std::{
    rc::Rc,
    task::{Context, Poll},
};
use sha2::{Sha256, Digest};
use crate::{
    services::DatabaseService,
    models::{ApiKey, User, SubscriptionTier},
};

/// Authentication context that gets added to request extensions
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub api_key: ApiKey,
    pub user: User,
    pub tier: SubscriptionTier,
}

/// Authentication middleware factory
pub struct AuthMiddleware {
    db_service: DatabaseService,
}

impl AuthMiddleware {
    pub fn new(db_service: DatabaseService) -> Self {
        Self { db_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            db_service: self.db_service.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    db_service: DatabaseService,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let db_service = self.db_service.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req.headers().get(AUTHORIZATION);
            
            let api_key = match extract_api_key(auth_header) {
                Some(key) => key,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Missing or invalid Authorization header. Expected 'Bearer <api_key>'"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Hash the API key for database lookup
            let key_hash = hash_api_key(&api_key);

            // Look up API key in database
            match db_service.get_api_key_with_details(&key_hash).await {
                Ok(Some((api_key_record, user, tier))) => {
                    // Add authentication context to request extensions
                    req.extensions_mut().insert(AuthContext {
                        api_key: api_key_record,
                        user,
                        tier,
                    });

                    // Continue to the next service
                    service.call(req).await
                }
                Ok(None) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid API key"
                        }));
                    Ok(req.into_response(response))
                }
                Err(e) => {
                    tracing::error!("Database error during authentication: {}", e);
                    let response = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal server error"
                        }));
                    Ok(req.into_response(response))
                }
            }
        })
    }
}

/// Extract API key from Authorization header
fn extract_api_key(auth_header: Option<&HeaderValue>) -> Option<String> {
    let auth_header = auth_header?;
    let auth_str = auth_header.to_str().ok()?;
    
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = auth_str.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        return None;
    }
    
    Some(token.to_string())
}

/// Hash an API key using SHA-256
fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_key() {
        use actix_web::http::header::HeaderValue;
        
        // Valid Bearer token
        let header = HeaderValue::from_static("Bearer test_api_key_123");
        assert_eq!(extract_api_key(Some(&header)), Some("test_api_key_123".to_string()));
        
        // Invalid format
        let header = HeaderValue::from_static("Basic test_api_key_123");
        assert_eq!(extract_api_key(Some(&header)), None);
        
        // Empty Bearer
        let header = HeaderValue::from_static("Bearer ");
        assert_eq!(extract_api_key(Some(&header)), None);
        
        // No header
        assert_eq!(extract_api_key(None), None);
    }

    #[test]
    fn test_hash_api_key() {
        let key = "test_key_123";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        let hash3 = hash_api_key("different_key");
        assert_ne!(hash1, hash3);
        
        // Hash should be 64 characters (256 bits / 4 bits per hex char)
        assert_eq!(hash1.len(), 64);
    }
}
