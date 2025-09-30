// wasm-wizard/src/middleware/pre_auth.rs

use crate::models::{ApiKey, SubscriptionTier, User};
use crate::services::DatabaseService;
use actix_web::body::EitherBody;
use actix_web::http::header::HeaderValue;
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, FromRequest, HttpMessage, HttpRequest, Result,
};
use chrono::Utc;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use sha2::{Digest, Sha256};
use std::rc::Rc;

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

// The middleware struct
#[derive(Clone)]
pub struct PreAuth {
    db_service: DatabaseService,
}

impl PreAuth {
    pub fn new(db_service: DatabaseService) -> Self {
        Self { db_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PreAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = PreAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PreAuthMiddleware {
            service: Rc::new(service),
            db_service: self.db_service.clone(),
        }))
    }
}

pub struct PreAuthMiddleware<S> {
    service: Rc<S>,
    db_service: DatabaseService,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub api_key: ApiKey,
    pub user: User,
    pub tier: SubscriptionTier,
}

// Implement FromRequest for AuthContext to resolve BorrowMutError
impl FromRequest for AuthContext {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthContext>().cloned() {
            Some(ctx) => ready(Ok(ctx)),
            None => ready(Err(ErrorUnauthorized("Authentication required"))),
        }
    }
}

impl<S, B> Service<ServiceRequest> for PreAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let db_service = self.db_service.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Extract Authorization header (Bearer format)
            let auth_header = req.headers().get("authorization");
            let api_key = match extract_api_key(auth_header) {
                Some(key) => key,
                None => {
                    let (request, _pl) = req.into_parts();
                    let response = actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Missing or invalid Authorization header. Expected 'Bearer <api_key>'"}))
                        .map_into_boxed_body();
                    return Ok(ServiceResponse::new(request, response).map_into_left_body());
                }
            };

            let key_hash = Sha256::digest(api_key.as_bytes());
            let key_hash_str = format!("{:x}", key_hash);

            match db_service.get_api_key_with_details(&key_hash_str).await {
                Ok(Some((mut api_key_record, user, tier))) => {
                    let now = Utc::now();

                    if let Some(expires_at) = api_key_record.expires_at {
                        if expires_at < now {
                            let (request, _pl) = req.into_parts();
                            let response = actix_web::HttpResponse::Unauthorized()
                                .json(serde_json::json!({"error": "API key has expired."}))
                                .map_into_boxed_body();
                            return Ok(ServiceResponse::new(request, response).map_into_left_body());
                        }
                    }

                    if let Err(err) = db_service.update_api_key_last_used(api_key_record.id).await {
                        tracing::error!("failed to update api key last_used_at: {err}");
                        let (request, _pl) = req.into_parts();
                        let response = actix_web::HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "An internal error occurred."}))
                            .map_into_boxed_body();
                        return Ok(ServiceResponse::new(request, response).map_into_left_body());
                    }

                    api_key_record.last_used_at = Some(now);

                    let auth_context = AuthContext {
                        api_key: api_key_record,
                        user,
                        tier,
                    };
                    req.extensions_mut().insert(auth_context);
                    service.call(req).await.map(|res| res.map_into_right_body())
                }
                Ok(None) => {
                    let (request, _pl) = req.into_parts();
                    let response = actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Invalid API key."}))
                        .map_into_boxed_body();
                    Ok(ServiceResponse::new(request, response).map_into_left_body())
                }
                Err(_) => {
                    let (request, _pl) = req.into_parts();
                    let response = actix_web::HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "An internal error occurred."}))
                        .map_into_boxed_body();
                    Ok(ServiceResponse::new(request, response).map_into_left_body())
                }
            }
        })
    }
}
