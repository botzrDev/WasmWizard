use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use thiserror::Error;

use crate::config::Config;
use crate::models::subscription_tier::SubscriptionTier;

/// Errors that can occur while executing a WebAssembly module.
#[derive(Debug, Error)]
pub enum WasmExecutionError {
    /// The provided module does not contain a valid WebAssembly header.
    #[error("Invalid WASM file format")]
    InvalidFormat,
    /// The module exceeds the configured size limits.
    #[error("WASM module size exceeds limit ({actual} bytes > {limit} bytes)")]
    ModuleTooLarge { actual: usize, limit: usize },
    /// The configured subscription tier prevents this execution due to memory constraints.
    #[error("WASM module exceeds the allowed memory limit for the current tier")]
    MemoryLimitExceeded,
    /// Execution took longer than the configured timeout.
    #[error("WASM execution timed out after {0:?}")]
    Timeout(Duration),
    /// Compilation failed.
    #[error("Failed to compile WASM module: {0}")]
    Compile(String),
    /// Instantiation failed.
    #[error("Failed to instantiate WASM module: {0}")]
    Instantiation(String),
    /// Execution trapped with a runtime error.
    #[error("WASM runtime error: {0}")]
    Runtime(String),
    /// Standard I/O failure when feeding or reading WASI pipes.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The host runtime encountered an unexpected error while spawning the execution task.
    #[error("Failed to join execution task: {0}")]
    Join(String),
}

/// Execute the supplied WebAssembly module bytes using Wasmer with WASI support.
///
/// The module is executed with the provided `input` piped to the WASI stdin. The
/// stdout of the module is captured and returned. Execution is bounded by the
/// configured timeout and memory limits for the current subscription tier.
pub async fn execute_wasm_bytes(
    wasm_bytes: &[u8],
    input: &str,
    config: &Config,
    tier: &SubscriptionTier,
) -> Result<String, WasmExecutionError> {
    if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] {
        return Err(WasmExecutionError::InvalidFormat);
    }

    if wasm_bytes.len() > config.max_wasm_size {
        return Err(WasmExecutionError::ModuleTooLarge {
            actual: wasm_bytes.len(),
            limit: config.max_wasm_size,
        });
    }

    let tier_memory_limit = (tier.max_memory_mb as usize).saturating_mul(1024 * 1024);
    let memory_limit = config.memory_limit.min(tier_memory_limit.max(1));
    if tier_memory_limit > 0 && wasm_bytes.len() > tier_memory_limit {
        return Err(WasmExecutionError::MemoryLimitExceeded);
    }
    if memory_limit == 0 {
        return Err(WasmExecutionError::MemoryLimitExceeded);
    }

    let timeout_secs = config
        .execution_timeout
        .min(tier.max_execution_time_seconds.max(1) as u64);
    let timeout_duration = Duration::from_secs(timeout_secs.max(1));

    let wasm_bytes = wasm_bytes.to_vec();
    let input = input.to_owned();

    let handle =
        tokio::task::spawn_blocking(move || run_wasm_blocking(&wasm_bytes, &input, memory_limit));

    match tokio::time::timeout(timeout_duration, handle).await {
        Ok(join_result) => match join_result {
            Ok(exec_result) => exec_result,
            Err(e) => Err(WasmExecutionError::Join(e.to_string())),
        },
        Err(_) => Err(WasmExecutionError::Timeout(timeout_duration)),
    }
}

fn run_wasm_blocking(
    wasm_bytes: &[u8],
    input: &str,
    memory_limit: usize,
) -> Result<String, WasmExecutionError> {
    use wasmer::{Instance, Module, Store};
    use wasmer::{StoreLimiter, StoreLimitsBuilder};
    use wasmer_wasi::{Pipe, WasiEnv, WasiState};

    let mut store = Store::default();
    let mut store_limits = StoreLimitsBuilder::new()
        .memory_size(memory_limit as u64)
        .instances(1)
        .tables(10)
        .build();

    store.set_limiter(|_| -> &mut dyn StoreLimiter { &mut store_limits });

    let module =
        Module::new(&store, wasm_bytes).map_err(|e| WasmExecutionError::Compile(e.to_string()))?;

    let stdin_pipe =
        Pipe::from_shared(Arc::new(Mutex::new(Cursor::new(input.as_bytes().to_vec()))));
    let stdout_pipe = Pipe::new();
    let stdout_reader = stdout_pipe.clone();
    let stderr_pipe = Pipe::new();

    let mut wasi_env = WasiState::new("wasm-wizard")
        .stdin(Box::new(stdin_pipe))
        .stdout(Box::new(stdout_pipe))
        .stderr(Box::new(stderr_pipe))
        .finalize(&mut store)
        .map_err(|e| WasmExecutionError::Instantiation(e.to_string()))?;

    let import_object = wasi_env
        .import_object(&mut store, &module)
        .map_err(|e| WasmExecutionError::Instantiation(e.to_string()))?;

    let instance = Instance::new(&mut store, &module, &import_object)
        .map_err(|e| WasmExecutionError::Instantiation(e.to_string()))?;

    wasi_env
        .start_instance(&mut store, &instance)
        .map_err(|e| WasmExecutionError::Runtime(e.to_string()))?;

    let mut output = String::new();
    stdout_reader.read_to_string(&mut output)?;

    Ok(output)
}
