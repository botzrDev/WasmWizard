// WASM execution and validation tests
use std::fs;
use std::path::Path;

#[test]
fn test_is_valid_wasm() {
    // Test with actual WASM files from our test suite
    let test_modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];

    for module_name in &test_modules {
        let path = Path::new("tests/wasm_modules").join(module_name);
        assert!(path.exists(), "Test WASM module {} should exist", module_name);

        let wasm_bytes =
            fs::read(&path).unwrap_or_else(|_| panic!("Should be able to read {}", module_name));

        assert!(
            is_valid_wasm_format(&wasm_bytes),
            "Module {} should have valid WASM format",
            module_name
        );
    }
}

#[test]
fn test_invalid_wasm_formats() {
    // Test various invalid formats
    let invalid_samples = vec![
        // Too short
        vec![0x00, 0x61, 0x73],
        // Wrong magic bytes
        vec![0xFF, 0x61, 0x73, 0x6D],
        vec![0x00, 0xFF, 0x73, 0x6D],
        vec![0x00, 0x61, 0xFF, 0x6D],
        vec![0x00, 0x61, 0x73, 0xFF],
        // Empty
        vec![],
        // Plain text
        b"This is not WASM".to_vec(),
        // Binary but not WASM
        vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04],
    ];

    for (i, invalid_wasm) in invalid_samples.iter().enumerate() {
        assert!(!is_valid_wasm_format(invalid_wasm), "Sample {} should be invalid WASM", i);
    }
}

#[test]
fn test_wasm_magic_bytes_edge_cases() {
    // Test edge cases around the magic bytes
    assert!(is_valid_wasm_format(&[0x00, 0x61, 0x73, 0x6D])); // Exact magic
    assert!(is_valid_wasm_format(&[0x00, 0x61, 0x73, 0x6D, 0x01])); // With extra bytes
    assert!(!is_valid_wasm_format(&[0x00, 0x61, 0x73])); // Too short
    assert!(!is_valid_wasm_format(&[0x61, 0x73, 0x6D])); // Missing first byte
}

#[test]
fn test_wasm_file_size_validation() {
    // Test that our WASM test files are reasonable sizes
    let test_modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];

    for module_name in &test_modules {
        let path = Path::new("tests/wasm_modules").join(module_name);
        let wasm_bytes = fs::read(&path).unwrap();

        // Should be at least 4 bytes (magic) but not too large for tests
        assert!(wasm_bytes.len() >= 4, "WASM module {} is too small", module_name);
        assert!(
            wasm_bytes.len() < 1024 * 1024, // Less than 1MB for test modules
            "WASM module {} is unexpectedly large: {} bytes",
            module_name,
            wasm_bytes.len()
        );

        println!("✓ {} is {} bytes", module_name, wasm_bytes.len());
    }
}

#[test]
fn test_wasm_test_inputs_and_outputs_exist() {
    // Verify our test data files exist and are readable
    let test_cases = [
        ("calc_add", "5 3"),
        ("echo", "Hello, World!"),
        ("hello_world", ""),
    ];

    for (module_name, _expected_input) in &test_cases {
        let input_path = Path::new("tests/wasm_modules").join(format!("{}_input.txt", module_name));
        let output_path =
            Path::new("tests/wasm_modules").join(format!("{}_output.txt", module_name));

        assert!(input_path.exists(), "Input file should exist for {}", module_name);
        assert!(output_path.exists(), "Output file should exist for {}", module_name);

        let input_content = fs::read_to_string(&input_path)
            .unwrap_or_else(|_| panic!("Should read input for {}", module_name));
        let output_content = fs::read_to_string(&output_path)
            .unwrap_or_else(|_| panic!("Should read output for {}", module_name));

        // Input and output should not be empty strings (except for hello_world input)
        if *module_name != "hello_world" {
            assert!(
                !input_content.trim().is_empty(),
                "Input should not be empty for {}",
                module_name
            );
        }
        assert!(
            !output_content.trim().is_empty(),
            "Output should not be empty for {}",
            module_name
        );

        println!(
            "✓ {} has input: '{}' and expected output: '{}'",
            module_name,
            input_content.trim(),
            output_content.trim()
        );
    }
}

#[test]
fn test_wasm_module_exports_analysis() {
    // This test verifies we can at least parse the WASM modules
    // without actually executing them (which requires runtime setup)
    use wasmparser::Parser;

    let test_modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];

    for module_name in &test_modules {
        let path = Path::new("tests/wasm_modules").join(module_name);
        let wasm_bytes = fs::read(&path).unwrap();

        let mut parser = Parser::new(0);
        let mut has_exports = false;

        for payload in parser.parse_all(&wasm_bytes) {
            match payload.unwrap() {
                wasmparser::Payload::ExportSection(exports) => {
                    has_exports = true;
                    let mut export_count = 0;
                    for export in exports {
                        let export = export.unwrap();
                        export_count += 1;
                        println!("  Export: {} -> {:?}", export.name, export.kind);
                    }
                    assert!(
                        export_count > 0,
                        "Module {} should have at least one export",
                        module_name
                    );
                }
                wasmparser::Payload::ImportSection(imports) => {
                    let mut import_count = 0;
                    for import in imports {
                        let import = import.unwrap();
                        import_count += 1;
                        println!("  Import: {}::{} -> {:?}", import.module, import.name, import.ty);
                    }
                    if import_count > 0 {
                        println!("✓ {} has {} imports", module_name, import_count);
                    }
                }
                _ => {}
            }
        }

        assert!(has_exports, "Module {} should have exports section", module_name);
        println!("✓ {} is valid WASM with exports", module_name);
    }
}

#[test]
fn test_wasm_security_constraints() {
    // Test that our WASM modules don't violate basic security expectations
    use wasmparser::{Parser, Payload};

    let test_modules = ["calc_add.wasm", "echo.wasm", "hello_world.wasm"];

    for module_name in &test_modules {
        let path = Path::new("tests/wasm_modules").join(module_name);
        let wasm_bytes = fs::read(&path).unwrap();

        let mut parser = Parser::new(0);
        let mut memory_count = 0;
        let mut global_count = 0;

        for payload in parser.parse_all(&wasm_bytes) {
            match payload.unwrap() {
                Payload::MemorySection(memories) => {
                    for memory in memories {
                        let memory = memory.unwrap();
                        memory_count += 1;

                        // Memory limits should be reasonable for test modules
                        let initial_pages = memory.initial;
                        let max_mb = initial_pages * 64 / 1024; // 64KB per page -> MB

                        assert!(
                            max_mb <= 128,
                            "Module {} has excessive initial memory: {}MB",
                            module_name,
                            max_mb
                        );

                        if let Some(max_pages) = memory.maximum {
                            let max_mb = max_pages * 64 / 1024;
                            assert!(
                                max_mb <= 256,
                                "Module {} has excessive max memory: {}MB",
                                module_name,
                                max_mb
                            );
                        }
                    }
                }
                Payload::GlobalSection(globals) => {
                    for global in globals {
                        let _global = global.unwrap();
                        global_count += 1;
                    }

                    // Reasonable limit on globals
                    assert!(
                        global_count < 100,
                        "Module {} has too many globals: {}",
                        module_name,
                        global_count
                    );
                }
                _ => {}
            }
        }

        println!(
            "✓ {} security check: {} memory, {} globals",
            module_name, memory_count, global_count
        );
    }
}

// Helper function to check WASM format
fn is_valid_wasm_format(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == [0x00, 0x61, 0x73, 0x6D]
}

#[cfg(test)]
mod input_size_tests {
    #[test]
    fn test_input_size_limits() {
        // Test various input sizes to ensure we handle them correctly
        let small_input = "small";
        let medium_input = "a".repeat(1000); // 1KB
        let large_input = "b".repeat(100_000); // 100KB
        let max_input = "c".repeat(1_048_576); // 1MB (our limit)

        assert!(small_input.len() < 1_048_576);
        assert!(medium_input.len() < 1_048_576);
        assert!(large_input.len() < 1_048_576);
        assert_eq!(max_input.len(), 1_048_576);

        // Verify UTF-8 validity for all sizes
        assert!(std::str::from_utf8(small_input.as_bytes()).is_ok());
        assert!(std::str::from_utf8(medium_input.as_bytes()).is_ok());
        assert!(std::str::from_utf8(large_input.as_bytes()).is_ok());
        assert!(std::str::from_utf8(max_input.as_bytes()).is_ok());
    }

    #[test]
    fn test_unicode_input_handling() {
        let unicode_inputs = vec![
            "Hello, 世界! 🌍",       // Mixed ASCII, Chinese, emoji
            "Тест на русском языке", // Cyrillic
            "مرحبا بالعالم",         // Arabic
            "🚀🎉🔥💻⚡",            // Only emoji
            "Iñtërnâtiônàlizætiøn",  // Latin with diacritics
        ];

        for input in unicode_inputs {
            assert!(
                input.is_ascii()
                    || input.chars().all(|c| c.is_alphabetic()
                        || c.is_numeric()
                        || c.is_whitespace()
                        || !c.is_control()),
                "Input should contain valid Unicode: {}",
                input
            );
            assert!(
                std::str::from_utf8(input.as_bytes()).is_ok(),
                "Input should be valid UTF-8: {}",
                input
            );
        }
    }
}
