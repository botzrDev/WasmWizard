// src/config.rs
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub api_salt: String,
    pub wasm_temp_dir: String,
    pub max_wasm_size: usize,
    pub max_input_size: usize,
    pub execution_timeout: u64,
    pub memory_limit: usize,
    pub free_tier_rate_minute: u32,
    pub free_tier_rate_day: u32,
    pub basic_tier_rate_minute: u32,
    pub basic_tier_rate_day: u32,
    pub pro_tier_rate_minute: u32,
    pub pro_tier_rate_day: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("SERVER_PORT must be a valid port number"))?,
            api_salt: env::var("API_SALT")
                .map_err(|_| ConfigError::Missing("API_SALT"))?,
            wasm_temp_dir: env::var("WASM_TEMP_DIR")
                .unwrap_or_else(|_| "/tmp/wasm_modules".to_string()),
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
            free_tier_rate_minute: env::var("FREE_TIER_RATE_MINUTE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("FREE_TIER_RATE_MINUTE must be a valid number"))?,
            free_tier_rate_day: env::var("FREE_TIER_RATE_DAY")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("FREE_TIER_RATE_DAY must be a valid number"))?,
            basic_tier_rate_minute: env::var("BASIC_TIER_RATE_MINUTE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("BASIC_TIER_RATE_MINUTE must be a valid number"))?,
            basic_tier_rate_day: env::var("BASIC_TIER_RATE_DAY")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("BASIC_TIER_RATE_DAY must be a valid number"))?,
            pro_tier_rate_minute: env::var("PRO_TIER_RATE_MINUTE")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("PRO_TIER_RATE_MINUTE must be a valid number"))?,
            pro_tier_rate_day: env::var("PRO_TIER_RATE_DAY")
                .unwrap_or_else(|_| "50000".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("PRO_TIER_RATE_DAY must be a valid number"))?,
        })
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
            return Err(ConfigError::Invalid("EXECUTION_TIMEOUT must be between 1 and 300 seconds"));
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
