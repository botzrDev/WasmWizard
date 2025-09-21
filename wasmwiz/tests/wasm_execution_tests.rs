use std::fs;
use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use wasm_wizard::config::{Config, Environment};
use wasm_wizard::models::subscription_tier::SubscriptionTier;
use wasm_wizard::wasm::{execute_wasm_bytes, WasmExecutionError};

fn test_config() -> Config {
    Config {
        database_url: "postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev"
            .to_string(),
        redis_url: "redis://127.0.0.1:6379".to_string(),
        redis_enabled: false,
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        api_salt: "test-salt".to_string(),
        max_wasm_size: 10 * 1024 * 1024,
        max_input_size: 1024 * 1024,
        execution_timeout: 5,
        memory_limit: 64 * 1024 * 1024,
        log_level: "info".to_string(),
        environment: Environment::Development,
        auth_required: false,
        csp_report_uri: None,
        csp_enable_nonce: false,
    }
}

fn test_tier() -> SubscriptionTier {
    SubscriptionTier {
        id: Uuid::new_v4(),
        name: "Test".to_string(),
        max_executions_per_minute: 60,
        max_executions_per_day: 1_000,
        max_memory_mb: 64,
        max_execution_time_seconds: 5,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime")
}

#[test]
fn executes_fixture_modules_successfully() {
    let wasm_dir = Path::new("tests/wasm_modules");
    let config = test_config();
    let tier = test_tier();
    let rt = runtime();

    let cases = [
        ("calc_add.wasm", "calc_add_input.txt", "calc_add_output.txt"),
        ("echo.wasm", "echo_input.txt", "echo_output.txt"),
        ("hello_world.wasm", "hello_world_input.txt", "hello_world_output.txt"),
    ];

    for (module, input_file, expected_file) in cases {
        let wasm_bytes = fs::read(wasm_dir.join(module)).expect("module should be readable");
        let input = fs::read_to_string(wasm_dir.join(input_file)).expect("input should exist");
        let expected =
            fs::read_to_string(wasm_dir.join(expected_file)).expect("expected output missing");

        let output = rt
            .block_on(execute_wasm_bytes(&wasm_bytes, &input, &config, &tier))
            .expect("execution should succeed");

        assert_eq!(output.trim(), expected.trim(), "{} should produce expected output", module);
    }
}

#[test]
fn rejects_invalid_wasm_modules() {
    let config = test_config();
    let tier = test_tier();
    let rt = runtime();

    let err = rt
        .block_on(execute_wasm_bytes(b"not wasm", "", &config, &tier))
        .expect_err("invalid module should be rejected");

    assert!(matches!(err, WasmExecutionError::InvalidFormat));
}

#[test]
fn enforces_module_size_limit() {
    let mut config = test_config();
    config.max_wasm_size = 4; // Force a very small limit to trigger the error.
    let tier = test_tier();
    let rt = runtime();

    let wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01];

    let err = rt
        .block_on(execute_wasm_bytes(&wasm, "", &config, &tier))
        .expect_err("module should exceed max size");

    assert!(matches!(err, WasmExecutionError::ModuleTooLarge { .. }));
}
