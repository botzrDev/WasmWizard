// src/models/api_payloads.rs
// Defines structs for API request and response bodies.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response payload for the /execute endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Request payload for generating API keys (example - adjust as needed).
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateApiKeyRequest {
    // e.g., pub user_id: Uuid, if an admin is generating for a specific user
    // For self-service, this might be empty or include account details.
}

/// Response payload for generating API keys.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateApiKeyResponse {
    pub api_key: String, // The plain text API key returned ONCE upon generation
    pub api_key_id: Uuid,
}
