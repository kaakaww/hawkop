//! Scan display models and helpers

use chrono::{DateTime, Utc};
use serde::Serialize;
use tabled::Tabled;

use crate::client::models::{AlertStats, ScanResult};

/// Scan display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct ScanDisplay {
    /// Scan ID
    #[tabled(rename = "SCAN ID")]
    pub id: String,

    /// Application name
    #[tabled(rename = "APP")]
    pub app: String,

    /// Environment
    #[tabled(rename = "ENV")]
    pub env: String,

    /// Scan status
    #[tabled(rename = "STATUS")]
    pub status: String,

    /// Findings summary (e.g., "3H1 5M0 2L0")
    #[tabled(rename = "FINDINGS")]
    pub findings: String,

    /// Scan duration (e.g., "12m 34s")
    #[tabled(rename = "DURATION")]
    pub duration: String,

    /// When the scan started (e.g., "2h ago")
    #[tabled(rename = "STARTED")]
    pub started: String,
}

impl From<ScanResult> for ScanDisplay {
    fn from(result: ScanResult) -> Self {
        let scan = &result.scan;

        // Format duration (API returns string)
        let duration = match &result.scan_duration {
            Some(secs_str) => {
                if let Ok(secs) = secs_str.parse::<f64>() {
                    if secs > 0.0 {
                        format_duration(secs)
                    } else {
                        "--".to_string()
                    }
                } else {
                    "--".to_string()
                }
            }
            None => "--".to_string(),
        };

        // Format started time (API returns timestamp as string)
        let started = if let Ok(ts) = scan.timestamp.parse::<i64>() {
            format_relative_time(ts)
        } else {
            "--".to_string()
        };

        // Format findings from alert_stats
        let findings = format_findings(&result);

        Self {
            id: scan.id.clone(),
            app: scan.application_name.clone(),
            env: scan.env.clone(),
            status: format_status(&scan.status),
            findings,
            duration,
            started,
        }
    }
}

impl From<&ScanResult> for ScanDisplay {
    fn from(result: &ScanResult) -> Self {
        ScanDisplay::from(result.clone())
    }
}

/// Scan overview for multi-section display (`scan <id>`)
///
/// Note: Replaced by the inline formatting in `show_pretty_overview()`, but kept
/// for backwards compatibility and potential future use.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct ScanOverview {
    pub scan: ScanResult,
}

#[allow(dead_code)]
impl ScanOverview {
    pub fn new(scan: ScanResult) -> Self {
        Self { scan }
    }

    /// Format as multi-section text output
    pub fn format_text(&self, scan_id: &str) -> String {
        let scan = &self.scan.scan;
        let mut output = String::new();

        // Header
        output.push_str(&format!("Scan: {} / {}\n", scan.application_name, scan.env));
        output.push_str("══════════════════════════════════════════════════════\n");

        // Metadata
        output.push_str(&format!("ID:       {}\n", scan.id));
        output.push_str(&format!("Status:   {}\n", format_status(&scan.status)));

        // Duration
        let duration = self
            .scan
            .scan_duration
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
            .map(format_duration)
            .unwrap_or_else(|| "--".to_string());
        output.push_str(&format!("Duration: {}\n", duration));

        // Started time
        let started = scan
            .timestamp
            .parse::<i64>()
            .map(format_relative_time)
            .unwrap_or_else(|_| "--".to_string());
        output.push_str(&format!("Started:  {}\n", started));

        // URLs scanned
        if let Some(url_count) = self.scan.url_count {
            output.push_str(&format!("URLs:     {}\n", url_count));
        }

        // Findings summary
        output.push_str("\nFindings\n");
        output.push_str("────────────────────────────────────────────────────\n");

        if let Some(ref stats) = self.scan.alert_stats {
            let (high_new, high_triaged, med_new, med_triaged, low_new, low_triaged) =
                count_findings_by_severity(stats);

            output.push_str(&format!(
                "  HIGH      {:>3} new    {:>3} triaged\n",
                high_new, high_triaged
            ));
            output.push_str(&format!(
                "  MEDIUM    {:>3} new    {:>3} triaged\n",
                med_new, med_triaged
            ));
            output.push_str(&format!(
                "  LOW       {:>3} new    {:>3} triaged\n",
                low_new, low_triaged
            ));
        } else {
            output.push_str("  No findings\n");
        }

        // Navigation hint
        output.push_str(&format!("\n→ hawkop scan view {} alerts\n", scan_id));

        output
    }
}

/// Format scan status for display (normalize case)
pub fn format_status(status: &str) -> String {
    match status.to_uppercase().as_str() {
        "STARTED" => "Running".to_string(),
        "COMPLETED" => "Complete".to_string(),
        "ERROR" => "Failed".to_string(),
        other => other.to_string(),
    }
}

/// Format duration in seconds to human-readable string
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;

    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    }
}

/// Format timestamp as relative time (e.g., "5m ago", "2h ago")
pub fn format_relative_time(timestamp_ms: i64) -> String {
    let scan_time = DateTime::from_timestamp_millis(timestamp_ms)
        .unwrap_or_else(|| DateTime::from_timestamp(timestamp_ms, 0).unwrap_or(Utc::now()));

    let now = Utc::now();
    let duration = now.signed_duration_since(scan_time);

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }

    if seconds < 60 {
        return format!("{}s ago", seconds);
    }

    let minutes = duration.num_minutes();
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return format!("{}h ago", hours);
    }

    let days = duration.num_days();
    if days < 7 {
        return format!("{}d ago", days);
    }

    let weeks = days / 7;
    if weeks < 4 {
        return format!("{}w ago", weeks);
    }

    // For older scans, show the date
    scan_time.format("%Y-%m-%d").to_string()
}

/// Format findings summary from alert_stats.
/// Format: "{new}{severity}{triaged}" e.g., "3H1 5M0 2L0"
/// - UNKNOWN status = new findings
/// - PROMOTED status = triaged findings
pub fn format_findings(result: &ScanResult) -> String {
    let alert_stats = match &result.alert_stats {
        Some(stats) => stats,
        None => return "--".to_string(),
    };

    // Collect counts by severity and status
    let mut high_new = 0u32;
    let mut high_triaged = 0u32;
    let mut medium_new = 0u32;
    let mut medium_triaged = 0u32;
    let mut low_new = 0u32;
    let mut low_triaged = 0u32;

    for status_stat in &alert_stats.alert_status_stats {
        let is_new = status_stat.alert_status == "UNKNOWN";
        let is_triaged = status_stat.alert_status == "PROMOTED"
            || status_stat.alert_status == "ACCEPTED"
            || status_stat.alert_status == "FALSE_POSITIVE";

        for (severity, count) in &status_stat.severity_stats {
            match severity.as_str() {
                "High" => {
                    if is_new {
                        high_new += count;
                    } else if is_triaged {
                        high_triaged += count;
                    }
                }
                "Medium" => {
                    if is_new {
                        medium_new += count;
                    } else if is_triaged {
                        medium_triaged += count;
                    }
                }
                "Low" => {
                    if is_new {
                        low_new += count;
                    } else if is_triaged {
                        low_triaged += count;
                    }
                }
                _ => {}
            }
        }
    }

    // If no findings at all, return "--"
    if high_new == 0
        && high_triaged == 0
        && medium_new == 0
        && medium_triaged == 0
        && low_new == 0
        && low_triaged == 0
    {
        return "--".to_string();
    }

    // Build findings string: "3H1 5M0 2L0"
    let mut parts = Vec::new();

    if high_new > 0 || high_triaged > 0 {
        parts.push(format!("{}H{}", high_new, high_triaged));
    }
    if medium_new > 0 || medium_triaged > 0 {
        parts.push(format!("{}M{}", medium_new, medium_triaged));
    }
    if low_new > 0 || low_triaged > 0 {
        parts.push(format!("{}L{}", low_new, low_triaged));
    }

    if parts.is_empty() {
        "--".to_string()
    } else {
        parts.join(" ")
    }
}

/// Count findings by severity from AlertStats
///
/// Note: Used by ScanOverview which is currently not in use but kept for
/// backwards compatibility.
#[allow(dead_code)]
fn count_findings_by_severity(stats: &AlertStats) -> (u32, u32, u32, u32, u32, u32) {
    let mut high_new = 0u32;
    let mut high_triaged = 0u32;
    let mut med_new = 0u32;
    let mut med_triaged = 0u32;
    let mut low_new = 0u32;
    let mut low_triaged = 0u32;

    for status_stat in &stats.alert_status_stats {
        let is_new = status_stat.alert_status == "UNKNOWN";
        let is_triaged = status_stat.alert_status == "PROMOTED"
            || status_stat.alert_status == "ACCEPTED"
            || status_stat.alert_status == "FALSE_POSITIVE"
            || status_stat.alert_status == "RISK_ACCEPTED";

        for (severity, count) in &status_stat.severity_stats {
            match severity.as_str() {
                "High" => {
                    if is_new {
                        high_new += count;
                    } else if is_triaged {
                        high_triaged += count;
                    }
                }
                "Medium" => {
                    if is_new {
                        med_new += count;
                    } else if is_triaged {
                        med_triaged += count;
                    }
                }
                "Low" => {
                    if is_new {
                        low_new += count;
                    } else if is_triaged {
                        low_triaged += count;
                    }
                }
                _ => {}
            }
        }
    }

    (
        high_new,
        high_triaged,
        med_new,
        med_triaged,
        low_new,
        low_triaged,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::models::{AlertStatusStats, Scan};
    use std::collections::HashMap;

    #[test]
    fn test_scan_display_from_scan_result() {
        let result = ScanResult {
            scan: Scan {
                id: "scan-123".to_string(),
                application_id: "app-456".to_string(),
                application_name: "TestApp".to_string(),
                env: "Production".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "1703721600000".to_string(), // 2023-12-28
                version: "5.0.0".to_string(),
                external_user_id: None,
            },
            scan_duration: Some("120".to_string()),
            url_count: Some(50),
            alert_stats: None,
            severity_stats: None,
            app_host: Some("https://testapp.example.com".to_string()),
            policy_name: None,
            tags: vec![],
            metadata: None,
        };

        let display = ScanDisplay::from(result);

        assert_eq!(display.id, "scan-123");
        assert_eq!(display.app, "TestApp");
        assert_eq!(display.env, "Production");
        assert_eq!(display.status, "Complete");
        assert_eq!(display.duration, "2m");
        assert_eq!(display.findings, "--");
    }

    #[test]
    fn test_scan_display_with_findings() {
        let mut high_severity = HashMap::new();
        high_severity.insert("High".to_string(), 3);

        let mut medium_severity = HashMap::new();
        medium_severity.insert("Medium".to_string(), 5);

        let result = ScanResult {
            scan: Scan {
                id: "scan-456".to_string(),
                application_id: "app-789".to_string(),
                application_name: "VulnApp".to_string(),
                env: "Staging".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "1703721600000".to_string(),
                version: "5.0.0".to_string(),
                external_user_id: None,
            },
            scan_duration: Some("300".to_string()),
            url_count: Some(100),
            alert_stats: Some(AlertStats {
                total_alerts: 8,
                unique_alerts: 8,
                alert_status_stats: vec![
                    AlertStatusStats {
                        alert_status: "UNKNOWN".to_string(),
                        total_count: 3,
                        severity_stats: high_severity,
                    },
                    AlertStatusStats {
                        alert_status: "UNKNOWN".to_string(),
                        total_count: 5,
                        severity_stats: medium_severity,
                    },
                ],
            }),
            severity_stats: None,
            app_host: Some("https://vulnapp.example.com".to_string()),
            policy_name: None,
            tags: vec![],
            metadata: None,
        };

        let display = ScanDisplay::from(result);

        assert_eq!(display.id, "scan-456");
        assert_eq!(display.duration, "5m");
        assert!(display.findings.contains("H")); // Has high findings
        assert!(display.findings.contains("M")); // Has medium findings
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(0.0), "0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60.0), "1m");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(125.0), "2m 5s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600.0), "1h");
        assert_eq!(format_duration(3660.0), "1h 1m");
        assert_eq!(format_duration(7320.0), "2h 2m");
    }

    #[test]
    fn test_format_findings_no_stats() {
        let result = ScanResult {
            scan: Scan {
                id: "scan-1".to_string(),
                application_id: "app-1".to_string(),
                application_name: "App".to_string(),
                env: "Dev".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "1703721600000".to_string(),
                version: "5.0.0".to_string(),
                external_user_id: None,
            },
            scan_duration: None,
            url_count: None,
            alert_stats: None,
            severity_stats: None,
            app_host: None,
            policy_name: None,
            tags: vec![],
            metadata: None,
        };

        assert_eq!(format_findings(&result), "--");
    }

    #[test]
    fn test_format_findings_with_triaged() {
        let mut high_new = HashMap::new();
        high_new.insert("High".to_string(), 2);

        let mut high_triaged = HashMap::new();
        high_triaged.insert("High".to_string(), 1);

        let result = ScanResult {
            scan: Scan {
                id: "scan-2".to_string(),
                application_id: "app-2".to_string(),
                application_name: "App".to_string(),
                env: "Prod".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "1703721600000".to_string(),
                version: "5.0.0".to_string(),
                external_user_id: None,
            },
            scan_duration: None,
            url_count: None,
            alert_stats: Some(AlertStats {
                total_alerts: 3,
                unique_alerts: 3,
                alert_status_stats: vec![
                    AlertStatusStats {
                        alert_status: "UNKNOWN".to_string(),
                        total_count: 2,
                        severity_stats: high_new,
                    },
                    AlertStatusStats {
                        alert_status: "PROMOTED".to_string(),
                        total_count: 1,
                        severity_stats: high_triaged,
                    },
                ],
            }),
            severity_stats: None,
            app_host: None,
            policy_name: None,
            tags: vec![],
            metadata: None,
        };

        let findings = format_findings(&result);
        assert!(findings.contains("H")); // Has high findings
        assert!(findings.contains("2")); // 2 new
        assert!(findings.contains("1")); // 1 triaged
    }
}
