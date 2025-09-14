//! # User Model
//!
//! This module defines the User data structure used throughout the application.
//! Users are the primary entities that own API keys and have associated usage metrics.
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE users (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     email VARCHAR(255) UNIQUE NOT NULL,
//!     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
//! );
//! ```
//!
//! ## Usage
//!
//! Users are created implicitly when they generate their first API key.
//! The email field is used for identification and potential future features
//! like notifications or account management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user in the WasmWiz system.
///
/// Users are identified by their email address and can have multiple API keys
/// associated with their account. User records are created automatically when
/// a new API key is generated for an email that doesn't exist yet.
///
/// # Examples
///
/// ```rust
/// use wasmwiz::models::user::User;
/// use chrono::Utc;
/// use uuid::Uuid;
///
/// let user = User {
///     id: Uuid::new_v4(),
///     email: "user@example.com".to_string(),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,

    /// User's email address - used as the primary identifier.
    /// Must be unique across all users in the system.
    pub email: String,

    /// Timestamp when the user account was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the user account was last updated.
    pub updated_at: DateTime<Utc>,
}
