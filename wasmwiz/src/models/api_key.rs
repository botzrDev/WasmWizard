//! # API Key Model
//!
//! This module defines the ApiKey data structure for managing authentication tokens.
//! API keys are the primary authentication mechanism for the Wasm Wizard API.
//!
//! ## Security Design
//!
//! - **Hashed Storage**: Only the SHA-256 hash of the API key is stored in the database
//! - **One-Time Display**: Plain text keys are shown only once upon generation
//! - **Expiration Support**: Keys can have optional expiration dates
//! - **Tier-Based Access**: Keys are associated with subscription tiers
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE api_keys (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     key_hash VARCHAR(64) UNIQUE NOT NULL, -- SHA-256 hash
//!     user_id UUID NOT NULL REFERENCES users(id),
//!     tier_id UUID NOT NULL REFERENCES subscription_tiers(id),
//!     is_active BOOLEAN DEFAULT TRUE,
//!     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     expires_at TIMESTAMP WITH TIME ZONE -- Optional expiration
//! );
//!
//! CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
//! CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
//! ```
//!
//! ## Usage Flow
//!
//! 1. User requests API key generation
//! 2. System generates random key and computes hash
//! 3. Plain key returned to user (one time only)
//! 4. Hash stored in database for authentication
//! 5. Subsequent requests use plain key for authentication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an API key for authentication and authorization.
///
/// API keys are the primary authentication mechanism for the Wasm Wizard API.
/// Each key is associated with a user and a subscription tier that determines
/// access levels and rate limits.
///
/// # Security Notes
///
/// - The `key_hash` field contains a SHA-256 hash of the actual API key
/// - The plain text API key is never stored and is shown to users only once
/// - Keys can be deactivated but not deleted for audit purposes
///
/// # Examples
///
/// ```rust
/// use wasm-wizard::models::api_key::ApiKey;
/// use chrono::Utc;
/// use uuid::Uuid;
///
/// let api_key = ApiKey {
///     id: Uuid::new_v4(),
///     key_hash: "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3".to_string(),
///     user_id: Uuid::new_v4(),
///     tier_id: Uuid::new_v4(), // References subscription tier
///     is_active: true,
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    /// Unique identifier for the API key.
    pub id: Uuid,

    /// SHA-256 hash of the actual API key used for authentication.
    /// The plain text key is never stored in the database.
    pub key_hash: String,

    /// Reference to the user who owns this API key.
    pub user_id: Uuid,

    /// Reference to the subscription tier for this key.
    /// Determines rate limits and feature access.
    pub tier_id: Uuid,

    /// Whether this API key is active and can be used for authentication.
    /// Inactive keys are rejected during authentication.
    pub is_active: bool,

    /// Timestamp when the API key was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the API key was last updated.
    pub updated_at: DateTime<Utc>,

    /// Optional expiration date for the API key.
    /// If None, the key never expires.
    pub expires_at: Option<DateTime<Utc>>,

    /// Timestamp when the API key was last used for authentication.
    /// Used for cleanup and analytics purposes.
    pub last_used_at: Option<DateTime<Utc>>,
}
