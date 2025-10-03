/// Askama template filters module
/// 
/// This module defines custom filters for Askama templates.
use chrono::{DateTime, Utc};

/// Format a DateTime for display
pub fn datetime(dt: &DateTime<Utc>) -> ::askama::Result<String> {
    Ok(dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

/// Format an Option<DateTime> for display
pub fn opt_datetime(dt: &Option<DateTime<Utc>>) -> ::askama::Result<String> {
    match dt {
        Some(d) => Ok(d.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        None => Ok("Never".to_string()),
    }
}

/// Length filter - returns the length of a sequence
pub fn length<T>(arr: &[T]) -> ::askama::Result<usize> {
    Ok(arr.len())
}
