//! # Configuration Management
//!
//! This module provides centralized configuration management for the Wasm Wizard application.
//! It supports environment-based configuration with sensible defaults for different deployment
//! environments (development, staging, production).
//!
//! ## Environment Variables
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `DATABASE_URL` | `postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev` | PostgreSQL connection string |
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
//! export DATABASE_URL="postgresql://user:pass@localhost/wasm-wizard"
//! export REDIS_URL="redis://localhost:6379"
//! export ENVIRONMENT="production"
//! export API_SALT="your-secure-salt-here"
//! ```

use serde::{Deserialize, Serialize};
use std::env;

/// Central configuration structure for the Wasm Wizard application.
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

    /// Google AdSense client ID for monetization (e.g., "ca-pub-XXXXXXXXXX")
    pub adsense_client_id: Option<String>,

    /// Enable advertisement display
    pub ads_enabled: bool,
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
    /// use wasm-wizard::config::Config;
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

        // Professional defaults based on environment
        let (default_host, default_log_level, default_auth) = match environment {
            Environment::Production => ("0.0.0.0", "info", true),
            Environment::Staging => ("0.0.0.0", "debug", true),
            Environment::Development => ("127.0.0.1", "debug", false),
        };

        // Default to local PostgreSQL for development
        let default_database_url = match environment {
            Environment::Development => {
                "postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev".to_string()
            }
            _ => env::var("DATABASE_URL").map_err(|_| {
                ConfigError::Missing("DATABASE_URL must be set for production/staging")
            })?,
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL").unwrap_or(default_database_url),
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
            api_salt: env::var("API_SALT")
                .unwrap_or_else(|_| "dev-salt-please-change-in-production".to_string()),
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
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| default_log_level.to_string()),
            environment,
            auth_required: env::var("AUTH_REQUIRED")
                .map(|v| v.parse().unwrap_or(default_auth))
                .unwrap_or(default_auth),
            csp_report_uri: env::var("CSP_REPORT_URI").ok(),
            csp_enable_nonce: env::var("CSP_ENABLE_NONCE")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            adsense_client_id: env::var("ADSENSE_CLIENT_ID").ok(),
            ads_enabled: env::var("ADS_ENABLED")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true), // Enable ads by default for free tier
        })
    }

    /// Returns true if the current environment is production.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wasm-wizard::config::Config;
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
    /// use wasm-wizard::config::Config;
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
    /// use wasm-wizard::config::Config;
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
            database_url: "postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev".to_string(),
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
            adsense_client_id: None,
            ads_enabled: true,
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

    /// A configuration value is invalid.
    #[error("Invalid configuration: {0}")]
    Invalid(&'static str),
}
