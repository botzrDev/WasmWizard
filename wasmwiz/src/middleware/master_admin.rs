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

#[derive(Debug, Clone, PartialEq)]
pub enum AdminRole {
    MasterAdmin,    // Full system access
    SystemAdmin,    // User and tier management
    SupportAdmin,   // View-only access and support functions
}

impl AdminRole {
    pub fn from_email(email: &str) -> Option<Self> {
        // Master admins - full system control
        if email == "admin@wasm-wizard.dev"
            || email == "master@wasm-wizard.dev"
            || email == "root@wasm-wizard.dev" {
            return Some(AdminRole::MasterAdmin);
        }

        // System admins - user and tier management
        if email.starts_with("admin.") && email.ends_with("@wasm-wizard.dev") {
            return Some(AdminRole::SystemAdmin);
        }

        // Support admins - view-only access
        if email.starts_with("support.") && email.ends_with("@wasm-wizard.dev") {
            return Some(AdminRole::SupportAdmin);
        }

        // Enterprise tier users get limited admin access
        None
    }

    pub fn can_access_endpoint(&self, path: &str) -> bool {
        match self {
            AdminRole::MasterAdmin => true, // Full access to everything
            AdminRole::SystemAdmin => {
                // Can manage users, API keys, and view analytics
                path.starts_with("/admin/users")
                    || path.starts_with("/admin/api-keys")
                    || path.starts_with("/admin/tiers")
                    || path.starts_with("/admin/analytics")
                    || path == "/admin"
                    || path == "/admin/"
            }
            AdminRole::SupportAdmin => {
                // Read-only access to user data and support functions
                (path.starts_with("/admin/users") && path.contains("/view"))
                    || (path.starts_with("/admin/api-keys") && path.contains("/view"))
                    || path.starts_with("/admin/support")
                    || path == "/admin"
                    || path == "/admin/"
            }
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            AdminRole::MasterAdmin => 3,
            AdminRole::SystemAdmin => 2,
            AdminRole::SupportAdmin => 1,
        }
    }
}

pub struct MasterAdminMiddleware {
    required_role: AdminRole,
}

impl MasterAdminMiddleware {
    pub fn new(required_role: AdminRole) -> Self {
        Self { required_role }
    }

    pub fn master_only() -> Self {
        Self::new(AdminRole::MasterAdmin)
    }

    pub fn system_admin_or_above() -> Self {
        Self::new(AdminRole::SystemAdmin)
    }

    pub fn support_admin_or_above() -> Self {
        Self::new(AdminRole::SupportAdmin)
    }
}

impl<S, B> Transform<S, ServiceRequest> for MasterAdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = MasterAdminMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MasterAdminMiddlewareService {
            service: Rc::new(service),
            required_role: self.required_role.clone(),
        }))
    }
}

pub struct MasterAdminMiddlewareService<S> {
    service: Rc<S>,
    required_role: AdminRole,
}

impl<S, B> Service<ServiceRequest> for MasterAdminMiddlewareService<S>
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
        let required_role = self.required_role.clone();

        Box::pin(async move {
            let path = req.path().to_string();

            // Check authentication and admin role
            let admin_check = {
                if let Some(auth_context) = req.extensions().get::<AuthContext>() {
                    if let Some(user_role) = AdminRole::from_email(&auth_context.user.email) {
                        // Check if user has sufficient role level
                        if user_role.level() >= required_role.level() {
                            // Check if user can access this specific endpoint
                            if user_role.can_access_endpoint(&path) {
                                Some((true, user_role, auth_context.user.email.clone()))
                            } else {
                                Some((false, user_role, auth_context.user.email.clone()))
                            }
                        } else {
                            Some((false, user_role, auth_context.user.email.clone()))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            match admin_check {
                Some((true, role, email)) => {
                    // User has admin access, log and continue
                    tracing::info!("Admin access granted: {} ({:?}) accessing {}", email, role, path);
                    service.call(req).await.map(|res| res.map_into_right_body())
                }
                Some((false, role, email)) => {
                    // User is admin but insufficient privileges
                    tracing::warn!("Admin access denied: {} ({:?}) attempted to access {}", email, role, path);
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "Insufficient privileges",
                            "message": format!(
                                "Your {} role does not have access to this endpoint. Required: {:?}",
                                match role {
                                    AdminRole::MasterAdmin => "Master Admin",
                                    AdminRole::SystemAdmin => "System Admin",
                                    AdminRole::SupportAdmin => "Support Admin",
                                },
                                required_role
                            ),
                            "user_role": match role {
                                AdminRole::MasterAdmin => "Master Admin",
                                AdminRole::SystemAdmin => "System Admin",
                                AdminRole::SupportAdmin => "Support Admin",
                            },
                            "required_role": match required_role {
                                AdminRole::MasterAdmin => "Master Admin",
                                AdminRole::SystemAdmin => "System Admin",
                                AdminRole::SupportAdmin => "Support Admin",
                            },
                            "endpoint": path
                        }));
                    Ok(req.into_response(response).map_into_left_body())
                }
                None => {
                    // No admin privileges at all
                    tracing::warn!("Non-admin user attempted to access admin endpoint: {}", path);
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "Admin access required",
                            "message": "This endpoint requires administrator privileges",
                            "endpoint": path,
                            "help": "Contact your system administrator if you believe you should have access"
                        }));
                    Ok(req.into_response(response).map_into_left_body())
                }
            }
        })
    }
}