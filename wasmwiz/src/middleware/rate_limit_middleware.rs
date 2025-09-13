// src/middleware/rate_limit_middleware.rs

use crate::middleware::distributed_rate_limit::RateLimitService;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

// Middleware for rate limiting
#[derive(Default)]
pub struct RateLimitMiddleware;

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self
    }
}

// Implement actix Transform trait for our middleware
impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

// The actual middleware service
pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        // Get rate limit service from app data
        if let Some(rate_limit_service) = req.app_data::<web::Data<RateLimitService>>() {
            let rate_limit_service = rate_limit_service.clone();
            let fut = async move {
                // Convert ServiceRequest to HttpRequest for rate limit check
                let http_req = req.request();

                // Check rate limit
                match rate_limit_service.check_request(http_req).await {
                    Ok(_) => {
                        // Rate limit passed, continue to next middleware
                        service.call(req).await
                    }
                    Err(err) => {
                        // Rate limit failed, return error response
                        Err(err.into())
                    }
                }
            };
            Box::pin(fut)
        } else {
            // No rate limit service found, pass through
            let fut = async move { service.call(req).await };
            Box::pin(fut)
        }
    }
}
