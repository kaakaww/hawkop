//! Reusable formatting utilities for CLI output
//!
//! This module provides common formatting functions for timestamps, durations,
//! and other display values used across multiple commands.

use chrono::{TimeZone, Utc};

/// Format Unix timestamp (milliseconds) to local date/time string.
///
/// Returns "N/A" if the timestamp is zero or invalid.
///
/// # Example output
/// `01/15/2025 14:30 PST`
pub fn format_timestamp_local(timestamp: &str) -> String {
    let millis: i64 = timestamp.parse().unwrap_or(0);
    if millis == 0 {
        return "N/A".to_string();
    }

    let secs = millis / 1000;
    match Utc.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt) => {
            let local = dt.with_timezone(&chrono::Local);
            let date_time = local.format("%m/%d/%Y %H:%M").to_string();
            let tz_abbrev = offset_to_tz_abbrev(local.offset().local_minus_utc());
            format!("{} {}", date_time, tz_abbrev)
        }
        _ => "N/A".to_string(),
    }
}

/// Convert UTC offset (seconds) to timezone abbreviation.
///
/// Maps common UTC offsets to standard timezone abbreviations. Falls back to
/// `UTC+N` format for uncommon offsets.
pub fn offset_to_tz_abbrev(offset_secs: i32) -> &'static str {
    let offset_hours = offset_secs / 3600;
    match offset_hours {
        -12 => "IDLW",
        -11 => "SST",
        -10 => "HST",
        -9 => "AKST",
        -8 => "PST",
        -7 => "MST",
        -6 => "CST",
        -5 => "EST",
        -4 => "AST",
        -3 => "ART",
        0 => "UTC",
        1 => "CET",
        2 => "EET",
        3 => "MSK",
        5 | 6 => "IST",
        8 => "CST", // China Standard Time
        9 => "JST",
        10 => "AEST",
        12 => "NZST",
        _ => {
            // Fall back to offset format for uncommon zones
            // This is a static str leak but only happens for rare offsets
            Box::leak(format!("UTC{:+}", offset_hours).into_boxed_str())
        }
    }
}

/// Format duration in seconds to human-readable string.
///
/// Returns "N/A" if the duration is zero or invalid.
///
/// # Example output
/// - `2h 15m 30s` (hours, minutes, seconds)
/// - `5m 10s` (minutes, seconds)
/// - `45s` (seconds only)
pub fn format_duration_seconds(seconds_str: &str) -> String {
    let secs: u64 = seconds_str.parse().unwrap_or(0);
    if secs == 0 {
        return "N/A".to_string();
    }

    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_local_valid() {
        // Jan 15, 2025 12:00:00 UTC in milliseconds
        let result = format_timestamp_local("1736942400000");
        // Should contain date and timezone, exact format depends on local TZ
        assert!(result.contains("01/15/2025"));
    }

    #[test]
    fn test_format_timestamp_local_zero() {
        assert_eq!(format_timestamp_local("0"), "N/A");
    }

    #[test]
    fn test_format_timestamp_local_invalid() {
        assert_eq!(format_timestamp_local("not-a-number"), "N/A");
    }

    #[test]
    fn test_offset_to_tz_abbrev_common() {
        assert_eq!(offset_to_tz_abbrev(-8 * 3600), "PST");
        assert_eq!(offset_to_tz_abbrev(-5 * 3600), "EST");
        assert_eq!(offset_to_tz_abbrev(0), "UTC");
        assert_eq!(offset_to_tz_abbrev(9 * 3600), "JST");
    }

    #[test]
    fn test_offset_to_tz_abbrev_uncommon() {
        // Uncommon offset should return UTC+N format
        let result = offset_to_tz_abbrev(7 * 3600);
        assert!(result.starts_with("UTC"));
    }

    #[test]
    fn test_format_duration_seconds_hours() {
        assert_eq!(format_duration_seconds("3661"), "1h 1m 1s");
        assert_eq!(format_duration_seconds("7200"), "2h 0m 0s");
    }

    #[test]
    fn test_format_duration_seconds_minutes() {
        assert_eq!(format_duration_seconds("125"), "2m 5s");
        assert_eq!(format_duration_seconds("60"), "1m 0s");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration_seconds("45"), "45s");
        assert_eq!(format_duration_seconds("1"), "1s");
    }

    #[test]
    fn test_format_duration_seconds_zero() {
        assert_eq!(format_duration_seconds("0"), "N/A");
    }

    #[test]
    fn test_format_duration_seconds_invalid() {
        assert_eq!(format_duration_seconds("not-a-number"), "N/A");
    }
}
