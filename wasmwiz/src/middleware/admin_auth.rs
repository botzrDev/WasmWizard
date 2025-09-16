use crate::middleware::pre_auth::AuthContext;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{
    rc::Rc,
    task::{Context, Poll},
};

pub struct AdminAuthMiddleware;

impl AdminAuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdminAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminAuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Check if user is authenticated and get admin status
            let admin_check = {
                if let Some(auth_context) = req.extensions().get::<AuthContext>() {
                    // Check if user has admin privileges
                    let is_admin = auth_context.user.email.ends_with("@wasm-wizard.dev")
                        || auth_context.user.email == "admin@example.com"
                        || auth_context.tier.name == "Enterprise";

                    if is_admin {
                        Some(true)
                    } else {
                        Some(false)
                    }
                } else {
                    None
                }
            };

            match admin_check {
                Some(true) => {
                    // User is admin, continue
                    service.call(req).await.map(|res| res.map_into_right_body())
                }
                Some(false) => {
                    // User is authenticated but not admin
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "Admin access required",
                            "message": "This endpoint requires administrator privileges"
                        }));
                    Ok(req.into_response(response).map_into_left_body())
                }
                None => {
                    // No auth context at all
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Authentication required",
                            "message": "Admin endpoints require authentication"
                        }));
                    Ok(req.into_response(response).map_into_left_body())
                }
            }
        })
    }
}