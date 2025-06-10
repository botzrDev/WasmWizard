// src/models/subscription_tier.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a subscription tier with defined limits.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionTier {
    pub id: Uuid,
    pub name: String,
    pub max_executions_per_minute: i32,
    pub max_executions_per_day: i32,
    pub max_memory_mb: i32,
    pub max_execution_time_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}