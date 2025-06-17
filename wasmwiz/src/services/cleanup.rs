// src/services/cleanup.rs
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, error, warn};
use crate::services::DatabaseService;

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
            
            // TODO: Add more cleanup tasks here, such as:
            // - Cleaning up expired API keys
            // - Archiving old user data
            // - Cleaning up temporary files that might have been missed
            
            info!("Daily cleanup tasks completed");
        }
    });
}

/// Clean up inactive API keys (mark as inactive, don't delete)
pub async fn cleanup_inactive_api_keys(
    _db_service: &DatabaseService,
    _inactive_days: i32,
) -> Result<u64, sqlx::Error> {
    let _cutoff_date = chrono::Utc::now() - chrono::Duration::days(_inactive_days as i64);
    
    // This would require tracking last_used_at in the api_keys table
    // For now, we'll just return 0 as a placeholder
    warn!("API key cleanup not implemented - requires last_used_at tracking");
    Ok(0)
}

/// Health check for the cleanup service
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
