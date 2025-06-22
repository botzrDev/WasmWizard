// src/config.rs
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub database_url: String,  // Always required, but can be SQLite
    pub redis_url: String,
    pub redis_enabled: bool,   // Enable Redis for rate limiting
    pub server_host: String,
    pub server_port: u16,
    pub api_salt: String,
    pub max_wasm_size: usize,
    pub max_input_size: usize,
    pub execution_timeout: u64,
    pub memory_limit: usize,
    pub log_level: String,
    pub environment: Environment,
    pub auth_required: bool,  // Feature flag for auth (off in development)
    pub csp_report_uri: Option<String>, // URI for CSP violation reports
    pub csp_enable_nonce: bool, // Whether to use nonce-based CSP
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = match env::var("ENVIRONMENT").as_deref() {
            Ok("production") => Environment::Production,
            Ok("staging") => Environment::Staging,
            _ => Environment::Development,
        };

        // Professional defaults based on environment
        let (default_host, default_log_level, default_auth) = match environment {
            Environment::Production => ("0.0.0.0", "info", true),
            Environment::Staging => ("0.0.0.0", "debug", true),
            Environment::Development => ("127.0.0.1", "debug", false),
        };

        // Default to local PostgreSQL for development
        let default_database_url = match environment {
            Environment::Development => "postgres://wasmwiz:wasmwiz@localhost:5432/wasmwiz_dev".to_string(),
            _ => env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL must be set for production/staging"))?
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or(default_database_url),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            redis_enabled: env::var("REDIS_ENABLED")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false), // Default to memory-based rate limiting
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| default_host.to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("SERVER_PORT must be a valid port number"))?,
            api_salt: env::var("API_SALT").unwrap_or_else(|_| "dev-salt-please-change-in-production".to_string()),
            max_wasm_size: env::var("MAX_WASM_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MAX_WASM_SIZE must be a valid number"))?,
            max_input_size: env::var("MAX_INPUT_SIZE")
                .unwrap_or_else(|_| "1048576".to_string()) // 1MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MAX_INPUT_SIZE must be a valid number"))?,
            execution_timeout: env::var("EXECUTION_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("EXECUTION_TIMEOUT must be a valid number"))?,
            memory_limit: env::var("MEMORY_LIMIT")
                .unwrap_or_else(|_| "134217728".to_string()) // 128MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MEMORY_LIMIT must be a valid number"))?,
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| default_log_level.to_string()),
            environment,
            auth_required: env::var("AUTH_REQUIRED")
                .map(|v| v.parse().unwrap_or(default_auth))
                .unwrap_or(default_auth),
            csp_report_uri: env::var("CSP_REPORT_URI").ok(),
            csp_enable_nonce: env::var("CSP_ENABLE_NONCE")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
        })
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.api_salt.len() < 16 {
            return Err(ConfigError::Invalid("API_SALT must be at least 16 characters"));
        }
        if self.server_port == 0 {
            return Err(ConfigError::Invalid("SERVER_PORT must be greater than 0"));
        }
        if self.max_wasm_size == 0 || self.max_wasm_size > 100 * 1024 * 1024 {
            return Err(ConfigError::Invalid("MAX_WASM_SIZE must be between 1 byte and 100MB"));
        }
        if self.max_input_size == 0 || self.max_input_size > 10 * 1024 * 1024 {
            return Err(ConfigError::Invalid("MAX_INPUT_SIZE must be between 1 byte and 10MB"));
        }
        if self.execution_timeout == 0 || self.execution_timeout > 300 {
            return Err(ConfigError::Invalid(
                "EXECUTION_TIMEOUT must be between 1 and 300 seconds",
            ));
        }
        if self.memory_limit < 1024 * 1024 || self.memory_limit > 1024 * 1024 * 1024 {
            return Err(ConfigError::Invalid("MEMORY_LIMIT must be between 1MB and 1GB"));
        }
        // Require Redis in production
        if self.is_production() {
            if self.redis_url.is_empty() {
                return Err(ConfigError::Missing("REDIS_URL must be set in production"));
            }
            if !self.redis_enabled {
                return Err(ConfigError::Invalid("REDIS_ENABLED must be true in production"));
            }
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://wasmwiz:wasmwiz@localhost:5432/wasmwiz_dev".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            redis_enabled: false,
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            api_salt: "dev-salt-please-change-in-production".to_string(),
            max_wasm_size: 10485760, // 10MB
            max_input_size: 1048576, // 1MB
            execution_timeout: 5,
            memory_limit: 134217728, // 128MB
            log_level: "debug".to_string(),
            environment: Environment::Development,
            auth_required: false,
            csp_report_uri: None,
            csp_enable_nonce: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("Invalid configuration: {0}")]
    Invalid(&'static str),
}
