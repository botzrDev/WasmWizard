//! # API Payload Models
//!
//! This module defines the data structures used for API request and response payloads.
//! These models ensure type-safe serialization and deserialization of JSON data
//! exchanged between clients and the WasmWiz API.
//!
//! ## Request/Response Flow
//!
//! ```text
//! Client Request → Deserialize → Handler → Business Logic → Serialize → Client Response
//!                      ↑                    ↓
//!                Validation            Database/Models
//! ```
//!
//! ## Examples
//!
//! ### Execute Endpoint
//! ```json
//! // Request (multipart/form-data)
//! {
//!   "wasm": "<wasm-binary-data>",
//!   "input": "Hello, WASM!"
//! }
//!
//! // Response
//! {
//!   "output": "Hello, WASM! Processed",
//!   "error": null
//! }
//! ```
//!
//! ### API Key Generation
//! ```json
//! // Request
//! {}
//!
//! // Response
//! {
//!   "api_key": "wasmwiz_abc123def456...",
//!   "api_key_id": "550e8400-e29b-41d4-a716-446655440000"
//! }
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response payload for WASM execution requests.
///
/// Contains the output from successful WASM execution or error details if execution failed.
/// Only one of `output` or `error` will be present in a successful response.
///
/// # Examples
///
/// ```rust
/// use wasmwiz::models::api_payloads::ExecuteResponse;
///
/// // Successful execution
/// let success = ExecuteResponse {
///     output: Some("Hello from WASM!".to_string()),
///     error: None,
/// };
///
/// // Failed execution
/// let failure = ExecuteResponse {
///     output: None,
///     error: Some("WASM execution timeout".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    /// The output string from successful WASM execution.
    /// `None` if execution failed or produced no output.
    pub output: Option<String>,

    /// Error message if WASM execution failed.
    /// `None` if execution was successful.
    pub error: Option<String>,
}

/// Request payload for generating new API keys.
///
/// Currently empty as API key generation is self-service, but can be extended
/// to include user identification or tier selection in the future.
///
/// # Examples
///
/// ```rust
/// use wasmwiz::models::api_payloads::GenerateApiKeyRequest;
///
/// // Currently empty - self-service generation
/// let request = GenerateApiKeyRequest {};
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateApiKeyRequest {
    // Future fields could include:
    // pub user_id: Uuid, // For admin-generated keys
    // pub tier: SubscriptionTier, // For tier selection
    // pub description: String, // For key labeling
}

/// Response payload containing the generated API key.
///
/// **Security Note**: The plain text API key is returned only once upon generation.
/// Store it securely as it cannot be retrieved again.
///
/// # Examples
///
/// ```rust
/// use wasmwiz::models::api_payloads::GenerateApiKeyResponse;
/// use uuid::Uuid;
///
/// let response = GenerateApiKeyResponse {
///     api_key: "wasmwiz_abc123def456...".to_string(),
///     api_key_id: Uuid::new_v4(),
/// };
///
/// // Store the api_key securely - it's shown only once!
/// println!("Your API key: {}", response.api_key);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct GenerateApiKeyResponse {
    /// The plain text API key for authentication.
    /// **Store this securely** - it will only be shown once!
    pub api_key: String,

    /// Unique identifier for the API key in the database.
    /// Use this for key management operations.
    pub api_key_id: Uuid,
}
