// src/models/usage_log.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a log entry for a Wasm module execution.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLog {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub execution_duration_ms: Option<i32>,
    pub memory_peak_mb: Option<f32>, // NUMERIC(5,2) in SQL maps well to f32
    pub status: String,              // "success", "execution_error", "time_limit_exceeded", etc.
    pub error_message: Option<String>,
    pub wasm_module_size_bytes: Option<i32>,
    pub input_size_bytes: Option<i32>,
}

impl UsageLog {
    pub fn new(api_key_id: Uuid, status: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            api_key_id,
            timestamp: Utc::now(),
            execution_duration_ms: None,
            memory_peak_mb: None,
            status,
            error_message: None,
            wasm_module_size_bytes: None,
            input_size_bytes: None,
        }
    }

    pub fn success(api_key_id: Uuid) -> Self {
        Self::new(api_key_id, "success".to_string())
    }

    pub fn error(api_key_id: Uuid, error_message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            api_key_id,
            timestamp: Utc::now(),
            execution_duration_ms: None,
            memory_peak_mb: None,
            status: "execution_error".to_string(),
            error_message: Some(error_message),
            wasm_module_size_bytes: None,
            input_size_bytes: None,
        }
    }

    pub fn with_execution_duration(mut self, duration_ms: i32) -> Self {
        self.execution_duration_ms = Some(duration_ms);
        self
    }

    pub fn with_file_sizes(mut self, wasm_size: i32, input_size: i32) -> Self {
        self.wasm_module_size_bytes = Some(wasm_size);
        self.input_size_bytes = Some(input_size);
        self
    }
}
