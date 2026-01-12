//! Common display utilities and helpers

use chrono::{DateTime, Utc};

/// Truncate string to max length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format timestamp string to ISO datetime (YYYY-MM-DDTHH:MM:SSZ)
pub fn format_as_iso_datetime(timestamp: &str) -> String {
    // Try parsing as ISO 8601 timestamp already
    if let Ok(dt) = timestamp.parse::<DateTime<Utc>>() {
        return dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    }

    // Try as Unix timestamp (milliseconds)
    if let Ok(ts_ms) = timestamp.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp_millis(ts_ms) {
            return dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        }
        // Try as seconds
        if let Some(dt) = DateTime::from_timestamp(ts_ms, 0) {
            return dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        }
    }

    // Return as-is if we can't parse it
    timestamp.to_string()
}
