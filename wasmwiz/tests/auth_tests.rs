// Comprehensive authentication tests
use wasmwiz::middleware::auth::{extract_api_key, hash_api_key};
use actix_web::http::{header, HeaderValue};

#[test]
fn test_extract_api_key_valid_bearer_token() {
    let header_value = HeaderValue::from_static("Bearer abc123def456");
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, Some("abc123def456".to_string()));
}

#[test]
fn test_extract_api_key_with_whitespace() {
    let header_value = HeaderValue::from_static("Bearer   abc123def456   ");
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, Some("abc123def456".to_string()));
}

#[test]
fn test_extract_api_key_no_bearer_prefix() {
    let header_value = HeaderValue::from_static("abc123def456");
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, None);
}

#[test]
fn test_extract_api_key_empty_token() {
    let header_value = HeaderValue::from_static("Bearer ");
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, None);
}

#[test]
fn test_extract_api_key_whitespace_only() {
    let header_value = HeaderValue::from_static("Bearer    ");
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, None);
}

#[test]
fn test_extract_api_key_none() {
    let result = extract_api_key(None);
    assert_eq!(result, None);
}

#[test]
fn test_extract_api_key_invalid_utf8() {
    // HeaderValue with invalid UTF-8 should return None
    let header_value = HeaderValue::from_bytes(b"Bearer \xFF\xFE").unwrap();
    let result = extract_api_key(Some(&header_value));
    assert_eq!(result, None);
}

#[test]
fn test_hash_api_key_consistency() {
    let api_key = "test_key_12345";
    let hash1 = hash_api_key(api_key);
    let hash2 = hash_api_key(api_key);

    // Same key should always produce same hash
    assert_eq!(hash1, hash2);

    // Hash should be SHA-256 hex string (64 characters)
    assert_eq!(hash1.len(), 64);
    assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_api_key_different_inputs() {
    let key1 = "test_key_1";
    let key2 = "test_key_2";
    let hash1 = hash_api_key(key1);
    let hash2 = hash_api_key(key2);

    // Different keys should produce different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_hash_api_key_empty_string() {
    let hash = hash_api_key("");
    assert_eq!(hash.len(), 64);
    // SHA-256 of empty string is known value
    assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn test_hash_api_key_unicode() {
    let unicode_key = "test_key_🔑_unicode";
    let hash = hash_api_key(unicode_key);
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_api_key_very_long() {
    let long_key = "a".repeat(10000);
    let hash = hash_api_key(&long_key);
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// Note: We can't easily test the private extract_api_key function
// without making it public or creating a separate test module
// This is a limitation we've worked around by testing the core logic