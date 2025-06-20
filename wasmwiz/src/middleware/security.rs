// src/middleware/security.rs
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header,
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};

pub struct SecurityHeadersMiddleware;

impl Default for SecurityHeadersMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityHeadersMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersService { service }))
    }
}

pub struct SecurityHeadersService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add security headers
            let headers = res.headers_mut();

            // HSTS (HTTP Strict Transport Security)
            headers.insert(
                header::HeaderName::from_static("strict-transport-security"),
                header::HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );

            // Content Security Policy
            headers.insert(
                header::HeaderName::from_static("content-security-policy"),
                header::HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data:; \
                     font-src 'self'; \
                     connect-src 'self'; \
                     frame-ancestors 'none'; \
                     base-uri 'self'",
                ),
            );

            // X-Frame-Options
            headers.insert(
                header::HeaderName::from_static("x-frame-options"),
                header::HeaderValue::from_static("DENY"),
            );

            // X-Content-Type-Options
            headers.insert(
                header::HeaderName::from_static("x-content-type-options"),
                header::HeaderValue::from_static("nosniff"),
            );

            // X-XSS-Protection
            headers.insert(
                header::HeaderName::from_static("x-xss-protection"),
                header::HeaderValue::from_static("1; mode=block"),
            );

            // Referrer Policy
            headers.insert(
                header::HeaderName::from_static("referrer-policy"),
                header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Permissions Policy
            headers.insert(
                header::HeaderName::from_static("permissions-policy"),
                header::HeaderValue::from_static(
                    "camera=(), microphone=(), geolocation=(), \
                     gyroscope=(), magnetometer=(), usb=()",
                ),
            );

            Ok(res)
        })
    }
}
