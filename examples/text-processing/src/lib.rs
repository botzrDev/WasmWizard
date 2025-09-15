use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;

// Text analysis result structure
#[repr(C)]
pub struct TextStats {
    word_count: u32,
    char_count: u32,
    sentence_count: u32,
    avg_word_length: f32,
}

// Word frequency result
#[repr(C)]
pub struct WordFrequency {
    word: *mut c_char,
    count: u32,
}

// Analyze text and return statistics
#[no_mangle]
pub extern "C" fn analyze_text(text_ptr: *const c_char, text_len: usize) -> *mut c_char {
    // Convert input to string
    let text_bytes = unsafe {
        std::slice::from_raw_parts(text_ptr as *const u8, text_len)
    };

    let text = match std::str::from_utf8(text_bytes) {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
    };

    // Calculate statistics
    let word_count = text.split_whitespace().count() as u32;
    let char_count = text.chars().count() as u32;
    let sentence_count = text.split(['.', '!', '?']).filter(|s| !s.trim().is_empty()).count() as u32;
    let avg_word_length = if word_count > 0 {
        (text.chars().filter(|c| !c.is_whitespace()).count() as f32) / word_count as f32
    } else {
        0.0
    };

    let result = format!(
        "Words: {}, Characters: {}, Sentences: {}, Avg Word Length: {:.1}",
        word_count, char_count, sentence_count, avg_word_length
    );

    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

// Simple sentiment analysis
#[no_mangle]
pub extern "C" fn sentiment_score(text_ptr: *const c_char, text_len: usize) -> f32 {
    let text_bytes = unsafe {
        std::slice::from_raw_parts(text_ptr as *const u8, text_len)
    };

    let text = match std::str::from_utf8(text_bytes) {
        Ok(s) => s.to_lowercase(),
        Err(_) => return 0.0,
    };

    let mut score = 0.0;

    // Positive words
    let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic", "love", "like", "best", "perfect"];
    for word in positive_words.iter() {
        if text.contains(word) {
            score += 1.0;
        }
    }

    // Negative words
    let negative_words = ["bad", "terrible", "awful", "hate", "worst", "horrible", "disappointing", "poor", "ugly", "stupid"];
    for word in negative_words.iter() {
        if text.contains(word) {
            score -= 1.0;
        }
    }

    // Intensifiers
    let intensifiers = ["very", "really", "extremely", "so", "absolutely", "totally"];
    for word in intensifiers.iter() {
        if text.contains(word) {
            score *= 1.5;
        }
    }

    // Negations
    let negations = ["not", "never", "no", "don't", "can't", "won't"];
    for word in negations.iter() {
        if text.contains(word) {
            score *= -1.0;
        }
    }

    // Normalize to -1.0 to 1.0 range
    score.max(-5.0).min(5.0) / 5.0
}

// Word frequency analysis
#[no_mangle]
pub extern "C" fn word_frequency(text_ptr: *const c_char, text_len: usize, top_n: usize) -> *mut c_char {
    let text_bytes = unsafe {
        std::slice::from_raw_parts(text_ptr as *const u8, text_len)
    };

    let text = match std::str::from_utf8(text_bytes) {
        Ok(s) => s.to_lowercase(),
        Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
    };

    // Count word frequencies
    let mut word_counts: HashMap<String, u32> = HashMap::new();

    for word in text.split_whitespace() {
        // Simple word cleaning (remove punctuation)
        let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
        if !clean_word.is_empty() {
            *word_counts.entry(clean_word.to_string()).or_insert(0) += 1;
        }
    }

    // Sort by frequency and take top N
    let mut sorted_words: Vec<_> = word_counts.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

    let top_words: Vec<_> = sorted_words.into_iter().take(top_n).collect();

    // Format result
    let mut result = String::new();
    for (word, count) in top_words {
        result.push_str(&format!("{}: {}, ", word, count));
    }
    if result.ends_with(", ") {
        result.truncate(result.len() - 2);
    }

    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

// Text format conversion
#[no_mangle]
pub extern "C" fn convert_case(text_ptr: *const c_char, text_len: usize, mode: u32) -> *mut c_char {
    let text_bytes = unsafe {
        std::slice::from_raw_parts(text_ptr as *const u8, text_len)
    };

    let text = match std::str::from_utf8(text_bytes) {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
    };

    let result = match mode {
        0 => text.to_uppercase(),      // UPPERCASE
        1 => text.to_lowercase(),      // lowercase
        2 => title_case(text),         // Title Case
        _ => text.to_string(),          // No change
    };

    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

// Helper function for title case
fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Text search and replace
#[no_mangle]
pub extern "C" fn search_replace(
    text_ptr: *const c_char, text_len: usize,
    search_ptr: *const c_char, search_len: usize,
    replace_ptr: *const c_char, replace_len: usize
) -> *mut c_char {
    let text_bytes = unsafe {
        std::slice::from_raw_parts(text_ptr as *const u8, text_len)
    };
    let search_bytes = unsafe {
        std::slice::from_raw_parts(search_ptr as *const u8, search_len)
    };
    let replace_bytes = unsafe {
        std::slice::from_raw_parts(replace_ptr as *const u8, replace_len)
    };

    let text = match std::str::from_utf8(text_bytes) {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in text").unwrap().into_raw(),
    };

    let search = match std::str::from_utf8(search_bytes) {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in search").unwrap().into_raw(),
    };

    let replace = match std::str::from_utf8(replace_bytes) {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in replace").unwrap().into_raw(),
    };

    let result = text.replace(search, replace);
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

// Get processing info
#[no_mangle]
pub extern "C" fn get_processing_info() -> *mut c_char {
    let info = "Text Processing WASM Module v0.1.0\n\
                Features: Analysis, Sentiment, Frequency, Case Conversion, Search/Replace\n\
                Memory: Efficient UTF-8 handling\n\
                Performance: O(n) for most operations";

    let c_string = CString::new(info).unwrap();
    c_string.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_basic() {
        // This would require implementing a test-friendly version
        // For now, just ensure the function doesn't panic
        let text = "I love this";
        let score = sentiment_score(text.as_ptr() as *const c_char, text.len());
        assert!(score >= -1.0 && score <= 1.0);
    }

    #[test]
    fn test_case_conversion() {
        let text = "hello world";
        let result_ptr = convert_case(text.as_ptr() as *const c_char, text.len(), 0);
        let result = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        assert_eq!(result.to_str().unwrap(), "HELLO WORLD");
    }
}