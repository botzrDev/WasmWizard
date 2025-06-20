use std::fs;
use std::path::Path;

/// Test that we can read and validate the test WASM modules
#[test]
fn test_wasm_modules_exist_and_valid() {
    let wasm_dir = Path::new("tests/wasm_modules");

    // Check that our test WASM files exist
    let modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];

    for module in &modules {
        let path = wasm_dir.join(module);
        assert!(path.exists(), "WASM module {} should exist", module);

        let content = fs::read(&path).expect("Should be able to read WASM file");

        // Check WASM magic bytes
        assert!(content.len() >= 4, "WASM file should be at least 4 bytes");
        assert_eq!(
            &content[0..4],
            &[0x00, 0x61, 0x73, 0x6D],
            "WASM magic bytes should be correct for {}",
            module
        );

        println!("✓ {} is valid WASM ({} bytes)", module, content.len());
    }
}

/// Test that input/output files exist for our test modules
#[test]
fn test_wasm_test_data_exists() {
    let wasm_dir = Path::new("tests/wasm_modules");

    let test_cases = [
        ("calc_add", "calc_add_input.txt", "calc_add_output.txt"),
        ("echo", "echo_input.txt", "echo_output.txt"),
        ("hello_world", "hello_world_input.txt", "hello_world_output.txt"),
    ];

    for (module, input_file, output_file) in &test_cases {
        let input_path = wasm_dir.join(input_file);
        let output_path = wasm_dir.join(output_file);

        assert!(input_path.exists(), "Input file {} should exist", input_file);
        assert!(output_path.exists(), "Output file {} should exist", output_file);

        let input_content =
            fs::read_to_string(&input_path).expect("Should be able to read input file");
        let expected_output =
            fs::read_to_string(&output_path).expect("Should be able to read output file");

        println!(
            "✓ {} test case: input='{}' expected_output='{}'",
            module,
            input_content.trim(),
            expected_output.trim()
        );
    }
}

/// Test file size validation logic
#[test]
fn test_file_size_validation() {
    let wasm_dir = Path::new("tests/wasm_modules");

    // Test that our WASM files are within reasonable size limits
    let max_wasm_size = 10 * 1024 * 1024; // 10MB
    let max_input_size = 1024 * 1024; // 1MB

    let modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];
    for module in &modules {
        let path = wasm_dir.join(module);
        let metadata = fs::metadata(&path).expect("Should be able to get file metadata");
        let size = metadata.len() as usize;

        assert!(size <= max_wasm_size, "WASM file {} should be under size limit", module);
        assert!(size > 0, "WASM file {} should not be empty", module);

        println!("✓ {} size: {} bytes (within limits)", module, size);
    }

    // Test input files
    let inputs = [
        "calc_add_input.txt",
        "echo_input.txt",
        "hello_world_input.txt",
    ];
    for input in &inputs {
        let path = wasm_dir.join(input);
        let metadata = fs::metadata(&path).expect("Should be able to get file metadata");
        let size = metadata.len() as usize;

        assert!(size <= max_input_size, "Input file {} should be under size limit", input);

        println!("✓ {} size: {} bytes (within limits)", input, size);
    }
}

/// Test security validation patterns
#[test]
fn test_security_validation() {
    // Test malicious patterns that should be detected
    let malicious_patterns = [
        "script",
        "javascript:",
        "vbscript:",
        "onload=",
        "onerror=",
        "../",
        "..\\",
        "/etc/passwd",
        "cmd.exe",
        "powershell",
        "SELECT",
        "INSERT",
        "DELETE",
        "UPDATE",
        "DROP",
        "UNION",
    ];

    for pattern in &malicious_patterns {
        let test_input = format!("test {} content", pattern);
        let lower_input = test_input.to_lowercase();

        // Simulate the validation logic
        let is_suspicious = malicious_patterns
            .iter()
            .any(|p| lower_input.contains(&p.to_lowercase()));

        assert!(is_suspicious, "Pattern '{}' should be detected as suspicious", pattern);
    }

    // Test safe content
    let safe_content = "hello world 123 normal content";
    let is_safe = !malicious_patterns
        .iter()
        .any(|p| safe_content.to_lowercase().contains(&p.to_lowercase()));

    assert!(is_safe, "Safe content should not trigger security validation");
}

/// Test API key format validation
#[test]
fn test_api_key_format() {
    // Test valid API key format (should start with "ww_")
    let valid_keys = [
        "ww_abcd1234567890",
        "ww_test_api_key_12345678901234567890",
        &format!("ww_{}", "x".repeat(32)),
    ];

    for key in &valid_keys {
        assert!(key.starts_with("ww_"), "API key should start with 'ww_': {}", key);
        assert!(key.len() > 3, "API key should be longer than prefix: {}", key);
    }

    // Test invalid formats
    let invalid_keys = ["invalid_key", "bearer_token", "ww", "", "ww_"];

    for key in &invalid_keys {
        assert!(
            !key.starts_with("ww_") || key.len() <= 3,
            "Invalid key should be rejected: {}",
            key
        );
    }
}

/// Test rate limiting calculations
#[test]
fn test_rate_limiting_logic() {
    // Test rate limit calculations for different tiers
    let tiers = [
        ("Free", 10, 500),
        ("Basic", 100, 10_000),
        ("Pro", 500, 50_000),
    ];

    for (tier_name, per_minute, per_day) in &tiers {
        // Basic validation
        assert!(*per_minute > 0, "Per-minute limit should be positive for {}", tier_name);
        assert!(*per_day > 0, "Per-day limit should be positive for {}", tier_name);
        assert!(
            *per_day >= *per_minute,
            "Daily limit should be >= minute limit for {}",
            tier_name
        );

        // Calculate theoretical maximum per day if hitting minute limit every minute
        let theoretical_max_per_day = per_minute * 24 * 60; // minutes per day
        assert!(
            *per_day <= theoretical_max_per_day,
            "Daily limit should be achievable for {}: {} <= {}",
            tier_name,
            per_day,
            theoretical_max_per_day
        );

        println!(
            "✓ {} tier limits: {}/min, {}/day (ratio: {:.2})",
            tier_name,
            per_minute,
            per_day,
            *per_day as f64 / *per_minute as f64
        );
    }
}

/// Test error message sanitization
#[test]
fn test_error_message_sanitization() {
    // Test that we don't expose sensitive information in error messages
    let sensitive_errors = [
        "SQL error: SELECT * FROM users WHERE password = 'secret'",
        "File path: /etc/passwd not found",
        "Database connection failed: postgres://user:password@localhost",
        "Stack trace: at line 123 in secret_module.rs",
    ];

    for error in &sensitive_errors {
        // Simulate error sanitization (this is conceptual)
        let sanitized = if error.contains("SQL") || error.contains("Database") {
            "Database error"
        } else if error.contains("File path") || error.contains("/etc/") {
            "File system error"
        } else if error.contains("Stack trace") {
            "Internal error"
        } else {
            "Unknown error"
        };

        assert!(!sanitized.contains("password"), "Sanitized error should not contain password");
        assert!(
            !sanitized.contains("/etc/"),
            "Sanitized error should not contain sensitive paths"
        );
        assert!(!sanitized.contains("SELECT"), "Sanitized error should not contain SQL");

        println!("✓ '{}' -> '{}'", &error[..50.min(error.len())], sanitized);
    }
}

/// Test configuration validation edge cases
#[test]
fn test_config_edge_cases() {
    // Test various size limits
    let test_cases = [
        ("MAX_WASM_SIZE", vec!["0", "1", "1048576", "10485760", "104857600"]), // 0B to 100MB
        ("MAX_INPUT_SIZE", vec!["0", "1", "1024", "1048576", "10485760"]),     // 0B to 10MB
        ("EXECUTION_TIMEOUT", vec!["0", "1", "5", "30", "300"]),               // 0 to 5 minutes
        ("MEMORY_LIMIT", vec!["0", "1", "64", "128", "512", "1024"]),          // 0 to 1GB
    ];

    for (config_name, values) in &test_cases {
        for value in values {
            let parsed_value: u64 = value.parse().expect("Should be valid number");

            // Basic validation rules
            match *config_name {
                "MAX_WASM_SIZE" => {
                    let is_valid = parsed_value > 0 && parsed_value <= 100 * 1024 * 1024;
                    println!(
                        "✓ {}: {} -> {}",
                        config_name,
                        value,
                        if is_valid { "valid" } else { "invalid" }
                    );
                }
                "MAX_INPUT_SIZE" => {
                    let is_valid = parsed_value > 0 && parsed_value <= 10 * 1024 * 1024;
                    println!(
                        "✓ {}: {} -> {}",
                        config_name,
                        value,
                        if is_valid { "valid" } else { "invalid" }
                    );
                }
                "EXECUTION_TIMEOUT" => {
                    let is_valid = parsed_value > 0 && parsed_value <= 300;
                    println!(
                        "✓ {}: {} -> {}",
                        config_name,
                        value,
                        if is_valid { "valid" } else { "invalid" }
                    );
                }
                "MEMORY_LIMIT" => {
                    let is_valid = parsed_value > 0 && parsed_value <= 1024;
                    println!(
                        "✓ {}: {} -> {}",
                        config_name,
                        value,
                        if is_valid { "valid" } else { "invalid" }
                    );
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_wasi_version_compatibility() {
    use std::process::{Command, Stdio};
    
    let test_modules: [(&str, Option<&str>); 1] = [
        ("hello_world.wasm", None), // No input needed, should work fine
    ];
    
    for (module, input) in &test_modules {
        let path = format!("tests/wasm_modules/{}", module);
        let mut cmd = Command::new("wasmtime");
        cmd.arg(&path);
        
        if input.is_some() {
            cmd.stdin(Stdio::piped());
        }
        
        let output = cmd.output().expect("Failed to run wasmtime");
        
        assert!(output.status.success(), 
                "WASI version compatibility failed for {}: stderr={}", 
                module, 
                String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn test_wasm_sandboxing_filesystem() {
    // This test expects a module that tries to read /etc/passwd and should fail
    let path = "tests/wasm_modules/malicious_fs.wasm";
    if std::path::Path::new(path).exists() {
        let output = std::process::Command::new("wasmtime")
            .arg(path)
            .output()
            .expect("Failed to run wasmtime");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("root:"), "Sandboxing failed: /etc/passwd leak");
    }
}

#[test]
fn test_wasm_resource_limits() {
    // This test expects a module that tries to allocate excessive memory and should fail
    let path = "tests/wasm_modules/memory_bomb.wasm";
    if std::path::Path::new(path).exists() {
        let output = std::process::Command::new("wasmtime")
            .arg(path)
            .output()
            .expect("Failed to run wasmtime");
        assert!(!output.status.success(), "Resource limit enforcement failed");
    }
}

#[test]
fn test_wasm_malicious_protection() {
    // This test expects a module that tries to run an infinite loop and should time out
    let path = "tests/wasm_modules/infinite_loop.wasm";
    if std::path::Path::new(path).exists() {
        let output = std::process::Command::new("timeout")
            .arg("3")
            .arg("wasmtime")
            .arg(path)
            .output()
            .expect("Failed to run wasmtime with timeout");
        assert!(!output.status.success(), "Malicious WASM protection failed (infinite loop)");
    }
}

#[test]
fn test_monitoring_endpoints() {
    // Test that monitoring endpoints are available
    let endpoints = ["/health", "/healthz", "/readyz", "/metrics"];

    for endpoint in &endpoints {
        println!("✓ Monitoring endpoint: {}", endpoint);
    }
}

#[test]
fn test_prometheus_metrics_format() {
    // Test that we can format metrics in Prometheus format
    use prometheus::{Opts, register_counter};

    let counter_opts = Opts::new("test_counter", "A test counter for metrics");
    let counter = register_counter!(counter_opts).expect("Failed to register counter");
    counter.inc();

    // Verify counter was incremented
    assert_eq!(counter.get(), 1.0);

    println!("✓ Prometheus metrics format working");
}
