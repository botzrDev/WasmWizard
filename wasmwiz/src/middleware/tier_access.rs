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

#[derive(Clone, Debug)]
pub enum RequiredTier {
    Free,
    Basic,
    Pro,
    Enterprise,
}

impl RequiredTier {
    pub fn from_tier_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "free" => RequiredTier::Free,
            "basic" => RequiredTier::Basic,
            "pro" => RequiredTier::Pro,
            "enterprise" => RequiredTier::Enterprise,
            _ => RequiredTier::Free,
        }
    }

    pub fn tier_level(&self) -> u8 {
        match self {
            RequiredTier::Free => 0,
            RequiredTier::Basic => 1,
            RequiredTier::Pro => 2,
            RequiredTier::Enterprise => 3,
        }
    }

    pub fn is_satisfied_by(&self, tier_name: &str) -> bool {
        let user_tier = RequiredTier::from_tier_name(tier_name);
        user_tier.tier_level() >= self.tier_level()
    }
}

pub struct TierAccessMiddleware {
    required_tier: RequiredTier,
}

impl TierAccessMiddleware {
    pub fn new(required_tier: RequiredTier) -> Self {
        Self { required_tier }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TierAccessMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = TierAccessMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TierAccessMiddlewareService {
            service: Rc::new(service),
            required_tier: self.required_tier.clone(),
        }))
    }
}

pub struct TierAccessMiddlewareService<S> {
    service: Rc<S>,
    required_tier: RequiredTier,
}

impl<S, B> Service<ServiceRequest> for TierAccessMiddlewareService<S>
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
        let required_tier = self.required_tier.clone();

        Box::pin(async move {
            // Check tier access before moving req
            let tier_check = {
                if let Some(auth_context) = req.extensions().get::<AuthContext>() {
                    if required_tier.is_satisfied_by(&auth_context.tier.name) {
                        Some(true)
                    } else {
                        Some(false)
                    }
                } else {
                    None
                }
            };

            match tier_check {
                Some(true) => {
                    // Tier access allowed, continue
                    service.call(req).await.map(|res| res.map_into_right_body())
                }
                Some(false) => {
                    // Authenticated but insufficient tier
                    let response = HttpResponse::PaymentRequired().json(serde_json::json!({
                        "error": "Upgrade required",
                        "message": format!(
                            "This feature requires {} tier or higher.",
                            match required_tier {
                                RequiredTier::Free => "Free",
                                RequiredTier::Basic => "Basic",
                                RequiredTier::Pro => "Pro",
                                RequiredTier::Enterprise => "Enterprise",
                            }
                        ),
                        "required_tier": match required_tier {
                            RequiredTier::Free => "Free",
                            RequiredTier::Basic => "Basic",
                            RequiredTier::Pro => "Pro",
                            RequiredTier::Enterprise => "Enterprise",
                        },
                        "upgrade_url": "/pricing"
                    }));
                    Ok(req.into_response(response).map_into_left_body())
                }
                None => {
                    // Not authenticated
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Authentication required",
                        "message": "Please provide a valid API key to access this feature"
                    }));
                    Ok(req.into_response(response).map_into_left_body())
                }
            }
        })
    }
}
