// src/services/cleanup.rs
use crate::services::DatabaseService;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Start a background task that periodically cleans up old data
pub fn start_cleanup_tasks(db_service: DatabaseService) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // Run daily

        loop {
            interval.tick().await;

            info!("Starting daily cleanup tasks...");

            // Clean up usage logs older than 30 days
            match db_service.cleanup_old_usage_logs(30).await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!("Cleaned up {} old usage log entries", deleted_count);
                    } else {
                        info!("No old usage logs to clean up");
                    }
                }
                Err(e) => {
                    error!("Failed to clean up old usage logs: {}", e);
                }
            }

            // Clean up expired and inactive API keys
            match cleanup_inactive_api_keys(&db_service, 90).await {
                Ok(deactivated_count) => {
                    if deactivated_count > 0 {
                        info!("Deactivated {} expired/inactive API keys", deactivated_count);
                    } else {
                        info!("No expired/inactive API keys to deactivate");
                    }
                }
                Err(e) => {
                    error!("Failed to clean up expired API keys: {}", e);
                }
            }

            // TODO: Add more cleanup tasks here, such as:
            // - Cleaning up old WASM modules that haven't been used
            // - Archiving old user data
            // - Cleaning up temporary files that might have been missed
            // - Rate limit table maintenance

            info!("Daily cleanup tasks completed");
        }
    });
}

/// Clean up inactive API keys (mark as inactive, don't delete)
#[allow(dead_code)]
pub async fn cleanup_inactive_api_keys(
    db_service: &DatabaseService,
    inactive_days: i32,
) -> Result<u64, sqlx::Error> {
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(inactive_days as i64);

    // Mark expired API keys as inactive
    let expired_result = sqlx::query!(
        "UPDATE api_keys SET is_active = FALSE, updated_at = NOW() 
         WHERE expires_at IS NOT NULL AND expires_at < NOW() AND is_active = TRUE"
    )
    .execute(&db_service.pool())
    .await?;

    let expired_count = expired_result.rows_affected();
    
    if expired_count > 0 {
        info!("Marked {} expired API keys as inactive", expired_count);
    }

    // Mark inactive API keys based on last_used_at (if tracking is implemented)
    // This would require updating last_used_at in authentication middleware
    let inactive_result = sqlx::query!(
        "UPDATE api_keys SET is_active = FALSE, updated_at = NOW()
         WHERE last_used_at IS NOT NULL AND last_used_at < $1 AND is_active = TRUE",
        cutoff_date
    )
    .execute(&db_service.pool())
    .await?;

    let inactive_count = inactive_result.rows_affected();
    
    if inactive_count > 0 {
        info!("Marked {} inactive API keys as inactive", inactive_count);
    }

    Ok(expired_count + inactive_count)
}

/// Health check for the cleanup service
#[allow(dead_code)]
pub async fn cleanup_health_check(db_service: &DatabaseService) -> bool {
    // Test that we can connect to the database
    match db_service.health_check().await {
        Ok(_) => true,
        Err(e) => {
            error!("Cleanup health check failed: {}", e);
            false
        }
    }
}
