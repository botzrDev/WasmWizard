use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn greet(name_ptr: *const c_char, name_len: usize) -> *mut c_char {
    // Convert the input pointer to a Rust string
    let name_bytes = unsafe {
        if name_ptr.is_null() {
            "World".as_bytes()
        } else {
            std::slice::from_raw_parts(name_ptr as *const u8, name_len)
        }
    };

    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) if !s.is_empty() => s,
        _ => "World",
    };

    // Create greeting message
    let greeting = format!("Hello, {}!", name);

    // Convert to C string and return pointer
    // Note: The caller is responsible for freeing this memory
    let c_string = CString::new(greeting).unwrap();
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn get_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    let c_string = CString::new(format!("Hello World WASM v{}", version)).unwrap();
    c_string.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_numbers() {
        assert_eq!(add_numbers(2, 3), 5);
        assert_eq!(add_numbers(-1, 1), 0);
        assert_eq!(add_numbers(0, 0), 0);
    }
}