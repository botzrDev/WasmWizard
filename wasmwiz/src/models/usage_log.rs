// src/models/usage_log.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a log entry for a Wasm module execution.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLog {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub execution_duration_ms: Option<i32>,
    pub memory_peak_mb: Option<f32>, // NUMERIC(5,2) in SQL maps well to f32
    pub status: String, // "success", "execution_error", "time_limit_exceeded", etc.
    pub error_message: Option<String>,
    pub wasm_module_size_bytes: Option<i32>,
    pub input_size_bytes: Option<i32>,
}