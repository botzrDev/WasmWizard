#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, web, App, http::StatusCode};
    use crate::middleware::pre_auth::{AuthContext, PreAuth};
    use crate::models::{ApiKey, User, SubscriptionTier};
    use uuid::Uuid;
    use chrono::Utc;

    #[actix_web::test]
    async fn test_execute_wasm_from_request_extractor() {
        // Create mock auth context
        let auth_context = AuthContext {
            api_key: ApiKey {
                id: Uuid::new_v4(),
                key_hash: "test_hash".to_string(),
                user_id: Uuid::new_v4(),
                tier_id: Uuid::new_v4(),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                expires_at: None,
                last_used_at: None,
            },
            user: User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            tier: SubscriptionTier {
                id: Uuid::new_v4(),
                name: "Test Tier".to_string(),
                max_executions_per_minute: 10,
                max_executions_per_day: 100,
                max_memory_mb: 128,
                max_execution_time_seconds: 5,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        };

        // Create a minimal test service
        let app = test::init_service(
            App::new()
                .service(
                    web::resource("/test")
                        .route(web::post().to(|auth: AuthContext| async move {
                            HttpResponse::Ok().json(serde_json::json!({
                                "user_email": auth.user.email,
                                "status": "success"
                            }))
                        }))
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::post()
            .uri("/test")
            .insert_header(("authorization", "Bearer test_key"))
            .to_request();

        // Insert the auth context into the request extensions
        req.extensions_mut().insert(auth_context);

        let resp = test::call_service(&app, req).await;
        
        // The test should pass if the FromRequest extractor works correctly
        // In a real scenario with middleware, this would be handled automatically
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
