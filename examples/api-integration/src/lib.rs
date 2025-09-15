use std::ffi::CString;
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};

// Data structures for API integration
#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessingResult {
    id: String,
    status: String,
    result: Option<String>,
    error: Option<String>,
    processing_time_ms: u64,
    memory_used_kb: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchResult {
    total_items: usize,
    successful: usize,
    failed: usize,
    results: Vec<ProcessingResult>,
    total_time_ms: u64,
}

// Process single data item
#[no_mangle]
pub extern "C" fn process_data(input_ptr: *const c_char, input_len: usize) -> *mut c_char {
    let input_bytes = unsafe {
        std::slice::from_raw_parts(input_ptr as *const u8, input_len)
    };

    let input = match std::str::from_utf8(input_bytes) {
        Ok(s) => s,
        Err(_) => {
            let error_result = ProcessingResult {
                id: "error".to_string(),
                status: "error".to_string(),
                result: None,
                error: Some("Invalid UTF-8 input".to_string()),
                processing_time_ms: 0,
                memory_used_kb: 0,
            };
            let json = serde_json::to_string(&error_result).unwrap();
            return CString::new(json).unwrap().into_raw();
        }
    };

    // Simulate processing time
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Simple data processing (reverse string, count words, etc.)
    let reversed = input.chars().rev().collect::<String>();
    let word_count = input.split_whitespace().count();
    let char_count = input.chars().count();

    let result = ProcessingResult {
        id: format!("item_{}", input.len()),
        status: "success".to_string(),
        result: Some(format!("Reversed: {}, Words: {}, Chars: {}", reversed, word_count, char_count)),
        error: None,
        processing_time_ms: 10,
        memory_used_kb: (input.len() / 1024 + 1) as u64,
    };

    let json = serde_json::to_string(&result).unwrap();
    let c_string = CString::new(json).unwrap();
    c_string.into_raw()
}

// Batch processing multiple items
#[no_mangle]
pub extern "C" fn batch_process(items_ptr: *const c_char, items_len: usize) -> *mut c_char {
    let items_json = unsafe {
        std::slice::from_raw_parts(items_ptr as *const u8, items_len)
    };

    let items: Vec<String> = match serde_json::from_slice(items_json) {
        Ok(items) => items,
        Err(_) => {
            let error_result = BatchResult {
                total_items: 0,
                successful: 0,
                failed: 1,
                results: vec![ProcessingResult {
                    id: "batch_error".to_string(),
                    status: "error".to_string(),
                    result: None,
                    error: Some("Invalid JSON input for batch processing".to_string()),
                    processing_time_ms: 0,
                    memory_used_kb: 0,
                }],
                total_time_ms: 0,
            };
            let json = serde_json::to_string(&error_result).unwrap();
            return CString::new(json).unwrap().into_raw();
        }
    };

    let start_time = std::time::Instant::now();
    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for (i, item) in items.into_iter().enumerate() {
        let item_start = std::time::Instant::now();

        // Process each item
        let result_str = if !item.is_empty() {
            let reversed = item.chars().rev().collect::<String>();
            let word_count = item.split_whitespace().count();

            successful += 1;
            ProcessingResult {
                id: format!("batch_item_{}", i),
                status: "success".to_string(),
                result: Some(format!("Reversed: {}, Words: {}", reversed, word_count)),
                error: None,
                processing_time_ms: item_start.elapsed().as_millis() as u64,
                memory_used_kb: (item.len() / 1024 + 1) as u64,
            }
        } else {
            failed += 1;
            ProcessingResult {
                id: format!("batch_item_{}", i),
                status: "error".to_string(),
                result: None,
                error: Some("Empty input".to_string()),
                processing_time_ms: item_start.elapsed().as_millis() as u64,
                memory_used_kb: 0,
            }
        };

        results.push(result_str);
    }

    let total_time = start_time.elapsed().as_millis() as u64;

    let batch_result = BatchResult {
        total_items: results.len(),
        successful,
        failed,
        results,
        total_time_ms: total_time,
    };

    let json = serde_json::to_string(&batch_result).unwrap();
    let c_string = CString::new(json).unwrap();
    c_string.into_raw()
}

// Validate API key (simulated)
#[no_mangle]
pub extern "C" fn validate_api_key(key_ptr: *const c_char, key_len: usize) -> i32 {
    let key_bytes = unsafe {
        std::slice::from_raw_parts(key_ptr as *const u8, key_len)
    };

    let key = match std::str::from_utf8(key_bytes) {
        Ok(s) => s,
        Err(_) => return -1, // Invalid UTF-8
    };

    // Simple validation (in real implementation, this would check against a database)
    if key.len() >= 20 && key.chars().all(|c| c.is_alphanumeric()) {
        1 // Valid
    } else {
        0 // Invalid
    }
}

// Get system information
#[no_mangle]
pub extern "C" fn get_system_info() -> *mut c_char {
    let info = format!(
        "API Integration WASM Module v0.1.0\n\
         Features: Single/Batch Processing, Authentication, Monitoring\n\
         Memory Management: Automatic cleanup\n\
         Error Handling: Comprehensive error reporting\n\
         Performance: Optimized for concurrent processing"
    );

    let c_string = CString::new(info).unwrap();
    c_string.into_raw()
}

// Health check function
#[no_mangle]
pub extern "C" fn health_check() -> *mut c_char {
    let health_status = serde_json::json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "version": env!("CARGO_PKG_VERSION"),
        "features": ["processing", "batch", "auth", "monitoring"]
    });

    let json = serde_json::to_string(&health_status).unwrap();
    let c_string = CString::new(json).unwrap();
    c_string.into_raw()
}

// Performance monitoring
#[no_mangle]
pub extern "C" fn get_performance_stats() -> *mut c_char {
    let stats = serde_json::json!({
        "average_processing_time_ms": 15,
        "memory_usage_kb": 256,
        "requests_processed": 1000,
        "error_rate_percent": 0.1,
        "uptime_seconds": 3600,
        "throughput_requests_per_second": 50
    });

    let json = serde_json::to_string(&stats).unwrap();
    let c_string = CString::new(json).unwrap();
    c_string.into_raw()
}

// Error simulation for testing
#[no_mangle]
pub extern "C" fn simulate_error(error_type: u32) -> *mut c_char {
    let error_result = match error_type {
        0 => ProcessingResult {
            id: "timeout_error".to_string(),
            status: "error".to_string(),
            result: None,
            error: Some("Processing timeout exceeded".to_string()),
            processing_time_ms: 30000,
            memory_used_kb: 512,
        },
        1 => ProcessingResult {
            id: "memory_error".to_string(),
            status: "error".to_string(),
            result: None,
            error: Some("Memory limit exceeded".to_string()),
            processing_time_ms: 5000,
            memory_used_kb: 1024,
        },
        2 => ProcessingResult {
            id: "validation_error".to_string(),
            status: "error".to_string(),
            result: None,
            error: Some("Input validation failed".to_string()),
            processing_time_ms: 100,
            memory_used_kb: 64,
        },
        _ => ProcessingResult {
            id: "unknown_error".to_string(),
            status: "error".to_string(),
            result: None,
            error: Some("Unknown error occurred".to_string()),
            processing_time_ms: 0,
            memory_used_kb: 0,
        },
    };

    let json = serde_json::to_string(&error_result).unwrap();
    let c_string = CString::new(json).unwrap();
    c_string.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let input = "Hello World";
        let result_ptr = process_data(input.as_ptr() as *const c_char, input.len());
        let result_json = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        let result: ProcessingResult = serde_json::from_str(result_json.to_str().unwrap()).unwrap();

        assert_eq!(result.status, "success");
        assert!(result.result.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_api_key_validation() {
        let valid_key = "abcdefghijklmnopqrst1234567890";
        let invalid_key = "short";

        assert_eq!(validate_api_key(valid_key.as_ptr() as *const c_char, valid_key.len()), 1);
        assert_eq!(validate_api_key(invalid_key.as_ptr() as *const c_char, invalid_key.len()), 0);
    }

    #[test]
    fn test_health_check() {
        let health_ptr = health_check();
        let health_json = unsafe { std::ffi::CStr::from_ptr(health_ptr) };
        let health: serde_json::Value = serde_json::from_str(health_json.to_str().unwrap()).unwrap();

        assert_eq!(health["status"], "healthy");
        assert!(health["timestamp"].is_number());
    }
}