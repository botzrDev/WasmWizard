// src/config.rs
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub database_url: Option<String>,  // Make optional for demo mode
    pub redis_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub api_salt: String,
    pub max_wasm_size: usize,
    pub max_input_size: usize,
    pub execution_timeout: u64,
    pub memory_limit: usize,
    pub log_level: String,
    pub environment: Environment,
    pub demo_mode: bool,  // Add demo mode flag
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Environment {
    Development,
    Demo,      // Add demo environment
    Staging,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = match env::var("ENVIRONMENT").as_deref() {
            Ok("production") => Environment::Production,
            Ok("staging") => Environment::Staging,
            Ok("demo") => Environment::Demo,
            _ => Environment::Development,
        };

        // Check for demo mode
        let demo_mode = env::var("DEMO_MODE").unwrap_or_else(|_| "false".to_string()) == "true" 
                     || environment == Environment::Demo;

        // Production-specific defaults
        let (default_host, default_log_level) = match environment {
            Environment::Production => ("0.0.0.0", "info"),
            Environment::Staging => ("0.0.0.0", "debug"),
            Environment::Demo => ("127.0.0.1", "debug"),
            Environment::Development => ("127.0.0.1", "debug"),
        };

        Ok(Config {
            database_url: if demo_mode {
                None  // No database required in demo mode
            } else {
                Some(env::var("DATABASE_URL")
                    .map_err(|_| ConfigError::Missing("DATABASE_URL"))?)
            },
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| default_host.to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("SERVER_PORT must be a valid port number"))?,
            api_salt: env::var("API_SALT").unwrap_or_else(|_| "demo-salt-12345".to_string()),
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
            demo_mode,
        })
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_demo(&self) -> bool {
        self.demo_mode || matches!(self.environment, Environment::Demo)
    }

    #[allow(dead_code)] // Utility method for future use
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

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("Invalid configuration: {0}")]
    Invalid(&'static str),
}
