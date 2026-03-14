//! Display models for environment command output
//!
//! These models format environment data for CLI display.

use chrono::{DateTime, Local, TimeZone, Utc};
use serde::Serialize;
use tabled::Tabled;

use crate::client::models::Environment;

/// Display model for environment list
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct EnvDisplay {
    #[tabled(rename = "NAME")]
    pub name: String,

    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "LAST SCAN")]
    pub last_scan: String,

    #[tabled(rename = "FINDINGS")]
    pub findings: String,
}

/// Format a Unix timestamp to a human-readable local time string.
///
/// The API may return timestamps in either seconds or milliseconds.
/// We detect which by checking if the value is too large for seconds.
fn format_timestamp(timestamp: i64) -> String {
    // If timestamp > year 3000 in seconds, it's probably milliseconds
    // Year 3000 in seconds ≈ 32503680000
    let timestamp_secs = if timestamp > 32_503_680_000 {
        timestamp / 1000 // Convert milliseconds to seconds
    } else {
        timestamp
    };

    let utc_time = Utc.timestamp_opt(timestamp_secs, 0).single();
    match utc_time {
        Some(utc) => {
            let local: DateTime<Local> = utc.into();
            local.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "-".to_string(),
    }
}

impl From<Environment> for EnvDisplay {
    fn from(env: Environment) -> Self {
        let last_scan = env
            .current_scan_summary
            .as_ref()
            .and_then(|s| s.timestamp)
            .map(format_timestamp)
            .unwrap_or_else(|| "-".to_string());

        let findings = env
            .current_scan_summary
            .as_ref()
            .and_then(|s| s.alert_stats.as_ref())
            .map(|stats| {
                if stats.high == 0 && stats.medium == 0 && stats.low == 0 {
                    "None".to_string()
                } else {
                    format!("{}H/{}M/{}L", stats.high, stats.medium, stats.low)
                }
            })
            .unwrap_or_else(|| "-".to_string());

        Self {
            name: env.environment_name,
            id: env.environment_id,
            last_scan,
            findings,
        }
    }
}

impl From<&Environment> for EnvDisplay {
    fn from(env: &Environment) -> Self {
        Self::from(env.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::models::{EnvAlertStats, EnvScanSummary};

    #[test]
    fn test_env_display_from_environment() {
        let env = Environment {
            environment_id: "env-123".to_string(),
            environment_name: "production".to_string(),
            latest_scan_type: Some("REST".to_string()),
            current_scan_summary: Some(EnvScanSummary {
                scan_id: Some("scan-123".to_string()),
                application_id: None,
                timestamp: Some(1706745600), // 2024-02-01 00:00:00 UTC
                config_hash: None,
                version: None,
                alert_stats: Some(EnvAlertStats {
                    high: 2,
                    medium: 5,
                    low: 10,
                }),
            }),
        };

        let display = EnvDisplay::from(env);
        assert_eq!(display.name, "production");
        assert_eq!(display.id, "env-123");
        // Timestamp should be formatted (exact format depends on local timezone)
        assert!(!display.last_scan.is_empty());
        assert_ne!(display.last_scan, "-");
        assert_eq!(display.findings, "2H/5M/10L");
    }

    #[test]
    fn test_env_display_no_findings() {
        let env = Environment {
            environment_id: "env-456".to_string(),
            environment_name: "staging".to_string(),
            latest_scan_type: None,
            current_scan_summary: None,
        };

        let display = EnvDisplay::from(env);
        assert_eq!(display.name, "staging");
        assert_eq!(display.last_scan, "-");
        assert_eq!(display.findings, "-");
    }

    #[test]
    fn test_env_display_with_zero_findings() {
        let env = Environment {
            environment_id: "env-789".to_string(),
            environment_name: "development".to_string(),
            latest_scan_type: Some("REST".to_string()),
            current_scan_summary: Some(EnvScanSummary {
                scan_id: Some("scan-456".to_string()),
                application_id: None,
                timestamp: Some(1706832000),
                config_hash: None,
                version: None,
                alert_stats: Some(EnvAlertStats {
                    high: 0,
                    medium: 0,
                    low: 0,
                }),
            }),
        };

        let display = EnvDisplay::from(env);
        assert_eq!(display.name, "development");
        assert_eq!(display.findings, "None"); // Zero findings shows "None"
    }
}
