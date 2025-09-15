// src/logging.rs
use crate::config::{Config, Environment};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init_logging(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    match config.environment {
        Environment::Production => {
            // JSON logging for production - structured for log aggregation
            tracing_subscriber::registry()
                .with(
                    fmt::layer()
                        .json()
                        .with_target(true)
                        .with_level(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_filter(env_filter),
                )
                .init();
        }
        Environment::Staging => {
            // JSON logging for staging
            tracing_subscriber::registry()
                .with(
                    fmt::layer()
                        .json()
                        .with_target(true)
                        .with_level(true)
                        .with_filter(env_filter),
                )
                .init();
        }
        Environment::Development => {
            // Pretty logging for development
            tracing_subscriber::registry()
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_level(true)
                        .with_filter(env_filter),
                )
                .init();
        }
    }

    info!(
        environment = ?config.environment,
        log_level = %config.log_level,
        "Logging initialized successfully"
    );

    Ok(())
}

// Production-safe logging macros that automatically include context
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!(
            target = "wasm-wizard",
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!(
            target = "wasm-wizard",
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!(
            target = "wasm-wizard",
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!(
            target = "wasm-wizard",
            $($arg)*
        )
    };
}
