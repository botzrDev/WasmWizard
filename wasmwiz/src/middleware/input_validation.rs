// src/middleware/input_validation.rs
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header,
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use tracing::warn;

pub struct InputValidationMiddleware;

impl Default for InputValidationMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl InputValidationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for InputValidationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InputValidationService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InputValidationService { service }))
    }
}

pub struct InputValidationService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for InputValidationService<S>
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
        // Validate request headers and content
        if let Err(validation_error) = validate_request(&req) {
            warn!("Request validation failed: {}", validation_error);
            // Continue with the request but log the validation issue
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

fn validate_request(req: &ServiceRequest) -> Result<(), String> {
    // Validate Content-Length header
    if let Some(content_length) = req.headers().get(header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                // 50MB limit for safety
                if length > 50 * 1024 * 1024 {
                    return Err("Request too large".to_string());
                }
            }
        }
    }

    // Validate User-Agent header for suspicious patterns
    if let Some(user_agent) = req.headers().get(header::USER_AGENT) {
        if let Ok(ua_str) = user_agent.to_str() {
            let suspicious_patterns = [
                "sqlmap",
                "nikto",
                "nmap",
                "masscan",
                "zap",
                "burp",
                "wget",
                "curl",
                "python-requests",
                "go-http-client",
            ];

            let ua_lower = ua_str.to_lowercase();
            for pattern in &suspicious_patterns {
                if ua_lower.contains(pattern) {
                    return Err(format!("Suspicious User-Agent: {}", pattern));
                }
            }
        }
    }

    // Validate for common attack patterns in query string
    let query = req.query_string();
    if !query.is_empty() {
        let malicious_patterns = [
            "script",
            "javascript:",
            "vbscript:",
            "onload=",
            "onerror=",
            "../",
            "..\\",
            "/etc/passwd",
            "cmd.exe",
            "powershell",
            "SELECT",
            "INSERT",
            "DELETE",
            "UPDATE",
            "DROP",
            "UNION",
        ];

        let query_lower = query.to_lowercase();
        for pattern in &malicious_patterns {
            if query_lower.contains(&pattern.to_lowercase()) {
                return Err(format!("Suspicious query parameter: {}", pattern));
            }
        }
    }

    Ok(())
}

/// Sanitize user input by removing or escaping potentially dangerous characters
#[allow(dead_code)]
pub fn sanitize_input(input: &str) -> String {
    input
        .replace('&', "&amp;") // Do this first to avoid double-encoding
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('\0', "") // Remove null bytes
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
}

/// Validate file extension
#[allow(dead_code)]
pub fn is_safe_filename(filename: &str) -> bool {
    let allowed_extensions = [".wasm"];
    let filename_lower = filename.to_lowercase();

    // Check for directory traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return false;
    }

    // Check for null bytes
    if filename.contains('\0') {
        return false;
    }

    // Check extension
    allowed_extensions
        .iter()
        .any(|ext| filename_lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        assert_eq!(
            sanitize_input("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(sanitize_input("normal text"), "normal text");
        assert_eq!(sanitize_input("text with \0 null byte"), "text with  null byte");
    }

    #[test]
    fn test_is_safe_filename() {
        assert!(is_safe_filename("test.wasm"));
        assert!(!is_safe_filename("../test.wasm"));
        assert!(!is_safe_filename("test.exe"));
        assert!(!is_safe_filename("test.wasm\0"));
    }
}
