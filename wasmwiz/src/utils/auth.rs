// src/utils/auth.rs
// Contains utility functions related to authentication and API keys.

use sha2::{Digest, Sha256};
use uuid::Uuid;

#[allow(dead_code)]
/// Hashes an API key using SHA-256.
/// This hash is stored in the database.
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[allow(dead_code)]
/// Generates a new unique API key (UUID v4).
pub fn generate_api_key() -> String {
    Uuid::new_v4().to_string()
}
