//! # WASM Module Model
//!
//! This module defines the WasmModule data structure for managing uploaded WebAssembly modules.
//! WASM modules can be stored, managed, and executed by users with appropriate access controls.
//!
//! ## Security Design
//!
//! - **User Isolation**: Each module is owned by a specific user
//! - **Hash Verification**: SHA-256 hash ensures module integrity
//! - **Size Limits**: Modules have size constraints to prevent abuse
//! - **Access Control**: Modules can be private or public
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE wasm_modules (
//!     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
//!     name TEXT NOT NULL,
//!     description TEXT,
//!     user_id UUID NOT NULL REFERENCES users(id),
//!     wasm_data BYTEA NOT NULL,
//!     size_bytes INTEGER NOT NULL,
//!     sha256_hash TEXT NOT NULL UNIQUE,
//!     upload_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     last_executed TIMESTAMP WITH TIME ZONE,
//!     execution_count INTEGER DEFAULT 0,
//!     is_public BOOLEAN DEFAULT FALSE,
//!     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
//! );
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a WebAssembly module stored in the system.
///
/// WASM modules are uploaded by users and can be executed multiple times.
/// Each module has metadata for tracking usage, security, and access control.
///
/// # Examples
///
/// ```rust,ignore
/// let module = WasmModule {
///     id: Uuid::new_v4(),
///     name: "fibonacci".to_string(),
///     description: Some("Calculates Fibonacci numbers".to_string()),
///     user_id: user_uuid,
///     wasm_data: wasm_binary_data,
///     size_bytes: wasm_binary_data.len() as i32,
///     sha256_hash: "abc123...".to_string(),
///     upload_time: Utc::now(),
///     last_executed: None,
///     execution_count: 0,
///     is_public: false,
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModule {
    /// Unique identifier for the WASM module.
    pub id: Uuid,

    /// Human-readable name for the module.
    pub name: String,

    /// Optional description of what the module does.
    pub description: Option<String>,

    /// Reference to the user who owns this module.
    pub user_id: Uuid,

    /// Binary WASM module data.
    pub wasm_data: Vec<u8>,

    /// Size of the WASM module in bytes.
    pub size_bytes: i32,

    /// SHA-256 hash of the WASM data for integrity verification.
    pub sha256_hash: String,

    /// Timestamp when the module was uploaded.
    pub upload_time: DateTime<Utc>,

    /// Timestamp of the last execution (if any).
    pub last_executed: Option<DateTime<Utc>>,

    /// Number of times this module has been executed.
    pub execution_count: i32,

    /// Whether this module can be accessed by other users.
    pub is_public: bool,

    /// Timestamp when the module record was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the module record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request payload for uploading a WASM module.
#[derive(Debug, Deserialize)]
pub struct UploadModuleRequest {
    /// Name for the uploaded module.
    pub name: String,

    /// Optional description.
    pub description: Option<String>,

    /// Whether to make the module publicly accessible.
    pub is_public: Option<bool>,
}

/// Response payload for successful module upload.
#[derive(Debug, Serialize)]
pub struct UploadModuleResponse {
    /// ID of the uploaded module.
    pub id: Uuid,

    /// Name of the uploaded module.
    pub name: String,

    /// Size of the uploaded module in bytes.
    pub size_bytes: i32,

    /// SHA-256 hash of the module.
    pub sha256_hash: String,

    /// Upload timestamp.
    pub upload_time: DateTime<Utc>,
}

/// Module metadata for listing endpoints (without binary data).
#[derive(Debug, Serialize)]
pub struct WasmModuleMeta {
    /// Module ID.
    pub id: Uuid,

    /// Module name.
    pub name: String,

    /// Module description.
    pub description: Option<String>,

    /// Module size in bytes.
    pub size_bytes: i32,

    /// SHA-256 hash.
    pub sha256_hash: String,

    /// Upload time.
    pub upload_time: DateTime<Utc>,

    /// Last execution time.
    pub last_executed: Option<DateTime<Utc>>,

    /// Execution count.
    pub execution_count: i32,

    /// Public accessibility flag.
    pub is_public: bool,
}

impl From<WasmModule> for WasmModuleMeta {
    fn from(module: WasmModule) -> Self {
        Self {
            id: module.id,
            name: module.name,
            description: module.description,
            size_bytes: module.size_bytes,
            sha256_hash: module.sha256_hash,
            upload_time: module.upload_time,
            last_executed: module.last_executed,
            execution_count: module.execution_count,
            is_public: module.is_public,
        }
    }
}
