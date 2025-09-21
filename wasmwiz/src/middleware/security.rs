// src/middleware/security.rs
use crate::config::Config;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage,
};
use base64::Engine;
use futures_util::future::LocalBoxFuture;
use rand::{thread_rng, Rng};
use std::future::{ready, Ready};
use std::sync::Arc;

pub struct SecurityHeadersMiddleware {
    config: Arc<Config>,
}

impl Default for SecurityHeadersMiddleware {
    fn default() -> Self {
        Self::new(Arc::new(Config::from_env().unwrap_or_default()))
    }
}

impl SecurityHeadersMiddleware {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    fn generate_nonce() -> String {
        let mut rng = thread_rng();
        let bytes: [u8; 16] = rng.gen();
        base64::engine::general_purpose::STANDARD.encode(bytes)
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
        ready(Ok(SecurityHeadersService {
            service,
            config: Arc::clone(&self.config),
        }))
    }
}

pub struct SecurityHeadersService<S> {
    service: S,
    config: Arc<Config>,
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
        // Generate nonce and add to request extensions before service call
        let nonce = if self.config.csp_enable_nonce {
            SecurityHeadersMiddleware::generate_nonce()
        } else {
            String::new()
        };

        if !nonce.is_empty() {
            req.extensions_mut().insert(nonce.clone());
        }

        let config = Arc::clone(&self.config);
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
            let mut csp = if config.is_production() {
                // Strict policy for production
                if nonce.is_empty() {
                    "default-src 'self'; \
                     script-src 'self'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data:; \
                     font-src 'self'; \
                     connect-src 'self'; \
                     frame-ancestors 'none'; \
                     base-uri 'self'"
                        .to_string()
                } else {
                    format!(
                        "default-src 'self'; \
                         script-src 'self' 'nonce-{}'; \
                         style-src 'self' 'nonce-{}'; \
                         img-src 'self' data:; \
                         font-src 'self'; \
                         connect-src 'self'; \
                         frame-ancestors 'none'; \
                         base-uri 'self'",
                        nonce, nonce
                    )
                }
            } else {
                // More permissive for development
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data:; \
                 font-src 'self'; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'"
                    .to_string()
            };

            if let Some(report_uri) = &config.csp_report_uri {
                csp.push_str(&format!("; report-uri {}", report_uri));
            }

            if let Ok(header_value) = header::HeaderValue::from_str(&csp) {
                headers.insert(
                    header::HeaderName::from_static("content-security-policy"),
                    header_value,
                );
            } else {
                tracing::error!("Failed to create CSP header value");
            }

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
