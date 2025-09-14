//! # Data Models
//!
//! This module defines all data structures used in the WasmWiz application,
//! including database entities, API payloads, and domain models.
//!
//! ## Model Categories
//!
//! ### Database Entities
//! - **`user`**: User accounts and authentication
//! - **`api_key`**: API keys for authentication and authorization
//! - **`subscription_tier`**: Subscription tiers and feature access
//! - **`usage_log`**: Request logging and analytics
//!
//! ### API Payloads
//! - **`api_payloads`**: Request/response structures for API endpoints
//!
//! ## Database Schema
//!
//! The models correspond to the following PostgreSQL tables:
//!
//! ```sql
//! -- Users
//! CREATE TABLE users (
//!     id UUID PRIMARY KEY,
//!     email VARCHAR(255) UNIQUE NOT NULL,
//!     created_at TIMESTAMP WITH TIME ZONE,
//!     updated_at TIMESTAMP WITH TIME ZONE
//! );
//!
//! -- API Keys
//! CREATE TABLE api_keys (
//!     id UUID PRIMARY KEY,
//!     key_hash VARCHAR(64) UNIQUE NOT NULL,
//!     user_id UUID REFERENCES users(id),
//!     tier_id UUID REFERENCES subscription_tiers(id),
//!     is_active BOOLEAN DEFAULT TRUE,
//!     created_at TIMESTAMP WITH TIME ZONE,
//!     updated_at TIMESTAMP WITH TIME ZONE
//! );
//!
//! -- Usage Logs
//! CREATE TABLE usage_logs (
//!     id UUID PRIMARY KEY,
//!     api_key_id UUID REFERENCES api_keys(id),
//!     endpoint VARCHAR(255),
//!     execution_duration_ms INTEGER,
//!     created_at TIMESTAMP WITH TIME ZONE
//! );
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use wasmwiz::models::{User, ApiKey, ExecuteResponse};
//!
//! // Database entities
//! let user = User {
//!     id: Uuid::new_v4(),
//!     email: "user@example.com".to_string(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//! };
//!
//! // API payloads
//! let response = ExecuteResponse {
//!     output: Some("Hello World!".to_string()),
//!     error: None,
//! };
//! ```
//!
//! ## Serialization
//!
//! All models implement `serde::Serialize` and `serde::Deserialize` for JSON
//! serialization, and `sqlx::FromRow` for database mapping.

pub mod api_key;
pub mod api_payloads;
pub mod subscription_tier;
pub mod usage_log;
pub mod user; // For request/response DTOs

// Re-export structs for easier access
pub use api_key::ApiKey;
pub use subscription_tier::SubscriptionTier;
pub use usage_log::UsageLog;
pub use user::User;
