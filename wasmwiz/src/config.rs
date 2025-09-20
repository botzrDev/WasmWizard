//! # Configuration Management
//!
//! This module provides centralized configuration management for the WasmWiz application.
//! It supports environment-based configuration with sensible defaults for different deployment
//! environments (development, staging, production).
//!
//! ## Environment Variables
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `DATABASE_URL` | `postgres://wasmwiz:wasmwiz@localhost:5432/wasmwiz_dev` | PostgreSQL connection string |
//! | `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection string for rate limiting |
//! | `REDIS_ENABLED` | `false` | Enable Redis-based rate limiting |
//! | `SERVER_HOST` | `127.0.0.1` (dev) / `0.0.0.0` (prod) | Server bind address |
//! | `SERVER_PORT` | `8080` | Server port |
//! | `API_SALT` | `dev-salt-please-change-in-production` | Salt for API key hashing |
//! | `MAX_WASM_SIZE` | `10485760` (10MB) | Maximum WASM module size |
//! | `MAX_INPUT_SIZE` | `1048576` (1MB) | Maximum input data size |
//! | `EXECUTION_TIMEOUT` | `5` | WASM execution timeout in seconds |
//! | `MEMORY_LIMIT` | `134217728` (128MB) | WASM memory limit in bytes |
//! | `LOG_LEVEL` | `debug` (dev) / `info` (prod) | Logging verbosity |
//! | `ENVIRONMENT` | `development` | Runtime environment |
//! | `AUTH_REQUIRED` | `false` (dev) / `true` (prod) | Enable authentication |
//! | `CSP_REPORT_URI` | - | URI for CSP violation reports |
//! | `CSP_ENABLE_NONCE` | `false` | Enable nonce-based CSP |
//!
//! ## Example
//!
//! ```bash
//! export DATABASE_URL="postgresql://user:pass@localhost/wasmwiz"
//! export REDIS_URL="redis://localhost:6379"
//! export ENVIRONMENT="production"
//! export API_SALT="your-secure-salt-here"
//! ```

use serde::{Deserialize, Serialize};
use std::{env, fs};

/// Central configuration structure for the WasmWiz application.
///
/// This struct contains all runtime configuration options loaded from environment variables
/// with sensible defaults. The configuration is validated on startup to ensure all required
/// values are present and within acceptable ranges.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// PostgreSQL database connection URL.
    /// Always required, supports both PostgreSQL and SQLite.
    pub database_url: String,

    /// Redis connection URL for distributed rate limiting and caching.
    pub redis_url: String,

    /// Whether Redis is enabled for rate limiting.
    /// When false, falls back to in-memory rate limiting.
    pub redis_enabled: bool,

    /// Server bind host address.
    pub server_host: String,

    /// Server bind port.
    pub server_port: u16,

    /// Salt used for hashing API keys.
    /// Must be at least 16 characters for security.
    pub api_salt: String,

    /// Maximum allowed size for uploaded WASM modules in bytes.
    pub max_wasm_size: usize,

    /// Maximum allowed size for input data in bytes.
    pub max_input_size: usize,

    /// Timeout for WASM execution in seconds.
    pub execution_timeout: u64,

    /// Memory limit for WASM execution in bytes.
    pub memory_limit: usize,

    /// Logging verbosity level (debug, info, warn, error).
    pub log_level: String,

    /// Current deployment environment.
    pub environment: Environment,

    /// Whether authentication is required for API endpoints.
    pub auth_required: bool,

    /// Optional URI for Content Security Policy violation reports.
    pub csp_report_uri: Option<String>,

    /// Whether to use nonce-based Content Security Policy.
    pub csp_enable_nonce: bool,
}

/// Deployment environment enumeration.
///
/// Determines default configuration values and behavior based on the target environment.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Environment {
    /// Development environment with relaxed security and debug features enabled.
    Development,

    /// Staging environment mirroring production but with some debug features.
    Staging,

    /// Production environment with full security and optimizations enabled.
    Production,
}

impl Config {
    /// Creates a new `Config` instance from environment variables.
    ///
    /// Loads configuration from environment variables with environment-specific defaults.
    /// Validates the configuration and returns an error if required values are missing
    /// or invalid.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Config)` with the loaded configuration, or `Err(ConfigError)` if
    /// the configuration is invalid or missing required values.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wasmwiz::config::Config;
    ///
    /// let config = Config::from_env()?;
    /// println!("Server will bind to {}:{}", config.server_host, config.server_port);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = match env::var("ENVIRONMENT").as_deref() {
            Ok("production") => Environment::Production,
            Ok("staging") => Environment::Staging,
            _ => Environment::Development,
        };

        let database_url_value = load_env_var("DATABASE_URL")?;
        let redis_url_value = load_env_var("REDIS_URL")?;
        let redis_enabled_value = load_env_var("REDIS_ENABLED")?;
        let server_host_value = load_env_var("SERVER_HOST")?;
        let server_port_value = load_env_var("SERVER_PORT")?;
        let api_salt_value = load_env_var("API_SALT")?;
        let max_wasm_size_value = load_env_var("MAX_WASM_SIZE")?;
        let max_input_size_value = load_env_var("MAX_INPUT_SIZE")?;
        let execution_timeout_value = load_env_var("EXECUTION_TIMEOUT")?;
        let memory_limit_value = load_env_var("MEMORY_LIMIT")?;
        let log_level_value = load_env_var("LOG_LEVEL")?;
        let auth_required_value = load_env_var("AUTH_REQUIRED")?;
        let csp_report_uri = load_env_var("CSP_REPORT_URI")?;
        let csp_enable_nonce_value = load_env_var("CSP_ENABLE_NONCE")?;

        // Professional defaults based on environment
        let (default_host, default_log_level, default_auth) = match environment {
            Environment::Production => ("0.0.0.0", "info", true),
            Environment::Staging => ("0.0.0.0", "debug", true),
            Environment::Development => ("127.0.0.1", "debug", false),
        };

        // Default to local PostgreSQL for development
        let database_url = match environment {
            Environment::Development => database_url_value.unwrap_or_else(|| {
                "postgres://wasmwiz:wasmwiz@localhost:5432/wasmwiz_dev".to_string()
            }),
            _ => database_url_value
                .ok_or(ConfigError::Missing("DATABASE_URL must be set for production/staging"))?,
        };

        Ok(Config {
            database_url,
            redis_url: redis_url_value.unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
            redis_enabled: redis_enabled_value
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false), // Default to memory-based rate limiting
            server_host: server_host_value.unwrap_or_else(|| default_host.to_string()),
            server_port: server_port_value
                .unwrap_or_else(|| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("SERVER_PORT must be a valid port number"))?,
            api_salt: api_salt_value
                .unwrap_or_else(|| "dev-salt-please-change-in-production".to_string()),
            max_wasm_size: max_wasm_size_value
                .unwrap_or_else(|| "10485760".to_string()) // 10MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MAX_WASM_SIZE must be a valid number"))?,
            max_input_size: max_input_size_value
                .unwrap_or_else(|| "1048576".to_string()) // 1MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MAX_INPUT_SIZE must be a valid number"))?,
            execution_timeout: execution_timeout_value
                .unwrap_or_else(|| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("EXECUTION_TIMEOUT must be a valid number"))?,
            memory_limit: memory_limit_value
                .unwrap_or_else(|| "134217728".to_string()) // 128MB
                .parse()
                .map_err(|_| ConfigError::Invalid("MEMORY_LIMIT must be a valid number"))?,
            log_level: log_level_value.unwrap_or_else(|| default_log_level.to_string()),
            environment,
            auth_required: auth_required_value
                .map(|v| v.parse().unwrap_or(default_auth))
                .unwrap_or(default_auth),
            csp_report_uri,
            csp_enable_nonce: csp_enable_nonce_value
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
        })
    }

    /// Returns true if the current environment is production.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wasmwiz::config::Config;
    ///
    /// let config = Config::from_env()?;
    /// if config.is_production() {
    ///     println!("Running in production mode");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    /// Returns true if the current environment is development.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wasmwiz::config::Config;
    ///
    /// let config = Config::from_env()?;
    /// if config.is_development() {
    ///     println!("Running in development mode - auth disabled");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    /// Validates the configuration values.
    ///
    /// Checks that all configuration values are within acceptable ranges and that
    /// required values are present for the current environment.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is valid, or `Err(ConfigError)` with
    /// details about what's invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wasmwiz::config::Config;
    ///
    /// let config = Config::from_env()?;
    /// config.validate()?;
    /// println!("Configuration is valid");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

/// Configuration error types.
///
/// Represents various configuration validation and loading errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required environment variable is missing.
    #[error("Missing required environment variable: {0}")]
    Missing(&'static str),

    /// Failed to read the file specified by a `*_FILE` environment variable.
    #[error("Failed to read environment file for {0}")]
    FileRead(&'static str, #[source] std::io::Error),

    /// A configuration value is invalid.
    #[error("Invalid configuration: {0}")]
    Invalid(&'static str),
}

fn load_env_var(key: &'static str) -> Result<Option<String>, ConfigError> {
    if let Ok(value) = env::var(key) {
        return Ok(Some(value));
    }

    let file_key = format!("{}_FILE", key);
    if let Ok(path) = env::var(&file_key) {
        let contents = fs::read_to_string(&path).map_err(|err| ConfigError::FileRead(key, err))?;
        let value = contents
            .trim_end_matches(|c| c == '\n' || c == '\r')
            .to_string();
        return Ok(Some(value));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::{io::Write, sync::Mutex};
    use uuid::Uuid;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn write_secret_file(prefix: &str, value: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("{}_{}", prefix, Uuid::new_v4()));
        let mut file = std::fs::File::create(&path).expect("failed to create secret file");
        writeln!(file, "{}", value).expect("failed to write secret value");
        path
    }

    #[test]
    fn loads_configuration_from_file_based_secrets() {
        let _guard = ENV_MUTEX.lock().unwrap();

        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("DATABASE_URL_FILE");
        std::env::remove_var("API_SALT");
        std::env::remove_var("API_SALT_FILE");
        std::env::remove_var("REDIS_URL");
        std::env::remove_var("REDIS_ENABLED");
        std::env::remove_var("ENVIRONMENT");

        let database_secret_path = write_secret_file(
            "wasmwiz_database_url",
            "postgres://wasmwiz:secret@postgres:5432/wasmwiz",
        );
        let api_salt_path = write_secret_file("wasmwiz_api_salt", "super-secure-api-salt-value");

        std::env::set_var("ENVIRONMENT", "production");
        std::env::set_var("DATABASE_URL_FILE", &database_secret_path);
        std::env::set_var("API_SALT_FILE", &api_salt_path);
        std::env::set_var("REDIS_URL", "redis://redis:6379");
        std::env::set_var("REDIS_ENABLED", "true");

        let config = Config::from_env().expect("config should load from file secrets");
        assert_eq!(config.database_url, "postgres://wasmwiz:secret@postgres:5432/wasmwiz",);
        assert_eq!(config.api_salt, "super-secure-api-salt-value");
        assert!(config.redis_enabled);
        assert!(config.is_production());
        config
            .validate()
            .expect("production config should validate");

        std::fs::remove_file(database_secret_path).ok();
        std::fs::remove_file(api_salt_path).ok();
        std::env::remove_var("DATABASE_URL_FILE");
        std::env::remove_var("API_SALT_FILE");
        std::env::remove_var("REDIS_URL");
        std::env::remove_var("REDIS_ENABLED");
        std::env::remove_var("ENVIRONMENT");
    }
}
