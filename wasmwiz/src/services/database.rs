// src/services/database.rs
use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use crate::models::{ApiKey, User, SubscriptionTier, UsageLog};

#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find an API key by its hash
    pub async fn find_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true"
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    /// Find a user by ID
    pub async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a subscription tier by ID
    pub async fn find_subscription_tier_by_id(&self, tier_id: Uuid) -> Result<Option<SubscriptionTier>> {
        let tier = sqlx::query_as::<_, SubscriptionTier>(
            "SELECT * FROM subscription_tiers WHERE id = $1"
        )
        .bind(tier_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tier)
    }

    /// Get API key with user and tier information
    pub async fn get_api_key_with_details(&self, key_hash: &str) -> Result<Option<(ApiKey, User, SubscriptionTier)>> {
        // First get the API key
        let api_key = match self.find_api_key_by_hash(key_hash).await? {
            Some(key) => key,
            None => return Ok(None),
        };

        // Get the user
        let user = match self.find_user_by_id(api_key.user_id).await? {
            Some(user) => user,
            None => return Ok(None),
        };

        // Get the subscription tier
        let tier = match self.find_subscription_tier_by_id(api_key.tier_id).await? {
            Some(tier) => tier,
            None => return Ok(None),
        };

        Ok(Some((api_key, user, tier)))
    }

    /// Create a new usage log entry
    pub async fn create_usage_log(&self, usage_log: &UsageLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO usage_logs (id, api_key_id, timestamp, execution_duration_ms, memory_peak_mb, status, error_message, wasm_module_size_bytes, input_size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(usage_log.id)
        .bind(usage_log.api_key_id)
        .bind(usage_log.timestamp)
        .bind(usage_log.execution_duration_ms)
        .bind(usage_log.memory_peak_mb)
        .bind(&usage_log.status)
        .bind(&usage_log.error_message)
        .bind(usage_log.wasm_module_size_bytes)
        .bind(usage_log.input_size_bytes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get usage count for API key within time period
    pub async fn get_usage_count_since(&self, api_key_id: Uuid, since: chrono::DateTime<chrono::Utc>) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usage_logs WHERE api_key_id = $1 AND timestamp >= $2"
        )
        .bind(api_key_id)
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Create a new API key
    pub async fn create_api_key(&self, api_key: &ApiKey) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, key_hash, user_id, tier_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(api_key.id)
        .bind(&api_key.key_hash)
        .bind(api_key.user_id)
        .bind(api_key.tier_id)
        .bind(api_key.is_active)
        .bind(api_key.created_at)
        .bind(api_key.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all API keys for a user
    pub async fn get_user_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let api_keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    /// Deactivate an API key
    pub async fn deactivate_api_key(&self, api_key_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(api_key_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    /// Clean up old usage logs (older than specified days)
    pub async fn cleanup_old_usage_logs(&self, days_old: i32) -> Result<u64, sqlx::Error> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_old as i64);
        
        let result = sqlx::query(
            "DELETE FROM usage_logs WHERE created_at < $1"
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }

    /// Get usage statistics for a user within a date range
    pub async fn get_user_usage_stats(
        &self,
        user_id: uuid::Uuid,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<UsageStats, sqlx::Error> {
        let stats = sqlx::query_as::<_, UsageStatsRow>(
            r#"
            SELECT 
                COUNT(*) as total_executions,
                SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_executions,
                AVG(CASE WHEN success THEN execution_duration_ms ELSE NULL END) as avg_execution_time,
                SUM(wasm_file_size_bytes) as total_wasm_bytes,
                SUM(input_size_bytes) as total_input_bytes
            FROM usage_logs ul
            JOIN api_keys ak ON ul.api_key_id = ak.id
            WHERE ak.user_id = $1 
            AND ul.created_at >= $2 
            AND ul.created_at <= $3
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(UsageStats {
            total_executions: stats.total_executions,
            successful_executions: stats.successful_executions,
            average_execution_time_ms: stats.avg_execution_time.map(|avg| avg as i32),
            total_wasm_bytes: stats.total_wasm_bytes,
            total_input_bytes: stats.total_input_bytes,
        })
    }
}

/// Database row for usage statistics
#[derive(sqlx::FromRow)]
struct UsageStatsRow {
    total_executions: i64,
    successful_executions: i64,
    avg_execution_time: Option<f64>,
    total_wasm_bytes: i64,
    total_input_bytes: i64,
}

/// Usage statistics structure
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_executions: i64,
    pub successful_executions: i64,
    pub average_execution_time_ms: Option<i32>,
    pub total_wasm_bytes: i64,
    pub total_input_bytes: i64,
}
