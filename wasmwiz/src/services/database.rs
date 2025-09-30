// src/services/database.rs
use crate::config::Config;
use crate::models::{ApiKey, SubscriptionTier, UsageLog, User};
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

/// Establishes a connection pool to the database
pub async fn establish_connection_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    let database_url = &config.database_url;

    PgPoolOptions::new()
        .max_connections(50) // Increased from 20 to 50
        .acquire_timeout(std::time::Duration::from_secs(30)) // Increased from 5 to 30 seconds
        .idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes idle timeout
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 minutes max lifetime
        .connect(database_url)
        .await
}

#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Update the last_used_at timestamp for an API key
    pub async fn update_api_key_last_used(&self, api_key_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(api_key_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Find an API key by its hash
    pub async fn find_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    /// Find a user by ID
    pub async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// Find a subscription tier by ID
    pub async fn find_subscription_tier_by_id(
        &self,
        tier_id: Uuid,
    ) -> Result<Option<SubscriptionTier>> {
        let tier =
            sqlx::query_as::<_, SubscriptionTier>("SELECT * FROM subscription_tiers WHERE id = $1")
                .bind(tier_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(tier)
    }

    /// Get API key with user and tier information
    pub async fn get_api_key_with_details(
        &self,
        key_hash: &str,
    ) -> Result<Option<(ApiKey, User, SubscriptionTier)>> {
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

    /// Create a new API key
    pub async fn create_api_key(&self, api_key: &ApiKey) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, key_hash, user_id, tier_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
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
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    /// Deactivate an API key
    pub async fn deactivate_api_key(&self, api_key_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(api_key_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;

        Ok(())
    }

    /// Clean up old usage logs (older than specified days)
    pub async fn cleanup_old_usage_logs(&self, days_old: i32) -> Result<u64, sqlx::Error> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        let result = sqlx::query("DELETE FROM usage_logs WHERE timestamp < $1")
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    // === ADMIN FUNCTIONALITY ===

    /// Count total users
    pub async fn count_users(&self) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    /// Count total API keys
    pub async fn count_api_keys(&self) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_keys WHERE is_active = true")
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    /// Count executions today
    pub async fn count_executions_today(&self) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usage_logs WHERE timestamp >= CURRENT_DATE"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    /// Count active users today
    pub async fn count_active_users_today(&self) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT u.id) FROM users u
             JOIN api_keys ak ON u.id = ak.user_id
             JOIN usage_logs ul ON ak.id = ul.api_key_id
             WHERE ul.timestamp >= CURRENT_DATE"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    /// Create a new user
    pub async fn create_user(&self, email: &str) -> Result<Uuid> {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())"
        )
        .bind(user_id)
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(user_id)
    }

    /// Get tier by name
    pub async fn get_tier_by_name(&self, name: &str) -> Result<Option<SubscriptionTier>> {
        let tier = sqlx::query_as::<_, SubscriptionTier>(
            "SELECT * FROM subscription_tiers WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(tier)
    }

    /// Create API key with user and tier IDs (admin version)
    pub async fn create_api_key_for_user(&self, user_id: Uuid, tier_id: Uuid) -> Result<String> {
        use crate::middleware::auth::hash_api_key;

        let api_key = format!("ww_{}", Uuid::new_v4().to_string().replace('-', ""));
        let key_hash = hash_api_key(&api_key);
        let api_key_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO api_keys (id, key_hash, user_id, tier_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, NOW(), NOW())
            "#
        )
        .bind(api_key_id)
        .bind(&key_hash)
        .bind(user_id)
        .bind(tier_id)
        .execute(&self.pool)
        .await?;

        Ok(api_key)
    }

    /// Update user tier
    pub async fn update_user_tier(&self, user_id: Uuid, tier_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE api_keys SET tier_id = $1, updated_at = NOW() WHERE user_id = $2"
        )
        .bind(tier_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all users with stats (placeholder - simplified)
    pub async fn get_all_users_with_stats(&self) -> Result<Vec<crate::handlers::admin::UserWithStats>> {
        use crate::handlers::admin::UserWithStats;

        let rows = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.created_at, u.updated_at,
                   st.name as tier_name,
                   COUNT(DISTINCT ak.id) as api_key_count,
                   COUNT(ul.id) as total_executions,
                   MAX(ul.timestamp) as last_activity
            FROM users u
            LEFT JOIN api_keys ak ON u.id = ak.user_id AND ak.is_active = true
            LEFT JOIN subscription_tiers st ON ak.tier_id = st.id
            LEFT JOIN usage_logs ul ON ak.id = ul.api_key_id
            GROUP BY u.id, u.email, u.created_at, u.updated_at, st.name
            ORDER BY u.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            users.push(UserWithStats {
                user: User {
                    id: row.id,
                    email: row.email,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                tier_name: row.tier_name.unwrap_or_else(|| "Free".to_string()),
                api_key_count: row.api_key_count.unwrap_or(0),
                total_executions: row.total_executions.unwrap_or(0),
                last_activity: row.last_activity,
            });
        }

        Ok(users)
    }

    /// Get all API keys with details (placeholder - simplified)
    pub async fn get_all_api_keys_with_details(&self) -> Result<Vec<crate::handlers::admin::ApiKeyWithDetails>> {
        use crate::handlers::admin::ApiKeyWithDetails;

        let rows = sqlx::query!(
            r#"
            SELECT ak.id, ak.key_hash, ak.user_id, ak.tier_id, ak.is_active, ak.created_at, ak.updated_at,
                   u.email as user_email,
                   st.name as tier_name,
                   COUNT(ul.id) as total_executions,
                   MAX(ul.timestamp) as last_used
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            JOIN subscription_tiers st ON ak.tier_id = st.id
            LEFT JOIN usage_logs ul ON ak.id = ul.api_key_id
            WHERE ak.is_active = true
            GROUP BY ak.id, ak.key_hash, ak.user_id, ak.tier_id, ak.is_active, ak.created_at, ak.updated_at,
                     u.email, st.name
            ORDER BY ak.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut api_keys = Vec::new();
        for row in rows {
            api_keys.push(ApiKeyWithDetails {
                api_key: ApiKey {
                    id: row.id,
                    key_hash: row.key_hash,
                    user_id: row.user_id,
                    tier_id: row.tier_id,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                user_email: row.user_email,
                tier_name: row.tier_name,
                total_executions: row.total_executions.unwrap_or(0),
                last_used: row.last_used,
            });
        }

        Ok(api_keys)
    }

    /// Get usage statistics (placeholder)
    pub async fn get_usage_statistics(&self) -> Result<crate::handlers::admin::UsageStats> {
        use crate::handlers::admin::UsageStats;

        let total_executions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM usage_logs")
            .fetch_one(&self.pool).await.unwrap_or(0);

        let executions_today = self.count_executions_today().await.unwrap_or(0);

        let executions_this_week = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usage_logs WHERE timestamp >= CURRENT_DATE - INTERVAL '7 days'"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        let executions_this_month = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usage_logs WHERE timestamp >= CURRENT_DATE - INTERVAL '30 days'"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        let success_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM usage_logs WHERE status = 'success'"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        let success_rate = if total_executions > 0 {
            (success_count as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let avg_execution_time = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT AVG(execution_duration_ms) FROM usage_logs WHERE execution_duration_ms IS NOT NULL"
        ).fetch_one(&self.pool).await?.unwrap_or(0.0);

        Ok(UsageStats {
            total_executions,
            executions_today,
            executions_this_week,
            executions_this_month,
            success_rate,
            average_execution_time: avg_execution_time,
        })
    }

    /// Get recent executions
    pub async fn get_recent_executions(&self, limit: i64) -> Result<Vec<crate::handlers::admin::RecentExecution>> {
        use crate::handlers::admin::RecentExecution;

        let rows = sqlx::query!(
            r#"
            SELECT ul.timestamp, ul.status, ul.execution_duration_ms, ul.error_message,
                   u.email as user_email, st.name as tier_name
            FROM usage_logs ul
            JOIN api_keys ak ON ul.api_key_id = ak.id
            JOIN users u ON ak.user_id = u.id
            JOIN subscription_tiers st ON ak.tier_id = st.id
            ORDER BY ul.timestamp DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut executions = Vec::new();
        for row in rows {
            executions.push(RecentExecution {
                timestamp: row.timestamp,
                user_email: row.user_email,
                tier_name: row.tier_name,
                status: row.status,
                execution_duration_ms: row.execution_duration_ms,
                error_message: row.error_message,
            });
        }

        Ok(executions)
    }

    /// Get all tiers
    pub async fn get_all_tiers(&self) -> Result<Vec<SubscriptionTier>> {
        let tiers = sqlx::query_as::<_, SubscriptionTier>(
            "SELECT * FROM subscription_tiers ORDER BY max_executions_per_minute ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tiers)
    }

    /// Create tier
    pub async fn create_tier(
        &self,
        name: &str,
        max_executions_per_minute: i32,
        max_executions_per_day: i32,
        max_memory_mb: i32,
        max_execution_time_seconds: i32,
    ) -> Result<Uuid> {
        let tier_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO subscription_tiers (id, name, max_executions_per_minute, max_executions_per_day,
                                          max_memory_mb, max_execution_time_seconds, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#
        )
        .bind(tier_id)
        .bind(name)
        .bind(max_executions_per_minute)
        .bind(max_executions_per_day)
        .bind(max_memory_mb)
        .bind(max_execution_time_seconds)
        .execute(&self.pool)
        .await?;
        Ok(tier_id)
    }
}
