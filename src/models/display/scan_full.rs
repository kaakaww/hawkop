//! Composite models for `scan get --detail full` output
//!
//! These models assemble data from multiple API calls into a single
//! self-contained JSON document optimized for AI agent consumption.
//! An agent can use this output to understand every finding from a scan,
//! including evidence, HTTP messages, remediation advice, and validation
//! commands — without needing any follow-up API calls.

use std::collections::HashMap;

use serde::Serialize;

/// Top-level output document for `scan get --detail full`
///
/// Contains everything an AI agent needs to understand and fix vulnerabilities
/// from a single scan. Designed to be consumed as a single JSON document.
#[derive(Debug, Clone, Serialize)]
pub struct ScanFullDetail {
    /// Schema version for forward-compatible parsing
    pub schema_version: String,

    /// Scan metadata (app, env, host, status, timing, policy)
    pub scan: ScanInfo,

    /// Aggregate finding counts by severity and triage status
    pub summary: FindingsSummary,

    /// All findings with full detail (paths, evidence, HTTP messages)
    pub findings: Vec<FindingFull>,

    /// Output metadata (generation time, API call stats)
    pub meta: OutputMeta,
}

/// Scan metadata extracted from the scan result
#[derive(Debug, Clone, Serialize)]
pub struct ScanInfo {
    /// Scan UUID
    pub id: String,

    /// Application UUID
    pub application_id: String,

    /// Application display name
    pub application_name: String,

    /// Environment name
    pub environment: String,

    /// Application host URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Scan status (COMPLETED, STARTED, ERROR)
    pub status: String,

    /// ISO 8601 completion timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,

    /// HawkScan version
    pub hawkscan_version: String,

    /// Policy display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,

    /// Email of the user who initiated the scan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Scan tags (branch, commit, etc.)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, String>,
}

/// Aggregate summary counts for the scan
#[derive(Debug, Clone, Serialize)]
pub struct FindingsSummary {
    /// Total number of findings (paths) across all plugins
    pub total_findings: usize,

    /// Counts by severity level
    pub by_severity: SeverityCounts,

    /// Counts by triage status
    pub by_status: StatusCounts,

    /// Number of URLs scanned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls_scanned: Option<u32>,
}

/// Finding counts per severity level
#[derive(Debug, Clone, Default, Serialize)]
pub struct SeverityCounts {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    #[serde(skip_serializing_if = "is_zero")]
    pub informational: usize,
}

/// Finding counts per triage status
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusCounts {
    /// New/untriaged findings (UNKNOWN status)
    pub new: usize,
    /// Assigned to a developer (PROMOTED status)
    #[serde(skip_serializing_if = "is_zero")]
    pub assigned: usize,
    /// Accepted risk (RISK_ACCEPTED status)
    #[serde(skip_serializing_if = "is_zero")]
    pub accepted: usize,
    /// False positive (FALSE_POSITIVE status)
    #[serde(skip_serializing_if = "is_zero")]
    pub false_positive: usize,
}

/// Full detail for a single finding/plugin type
#[derive(Debug, Clone, Serialize)]
pub struct FindingFull {
    /// Scanner plugin ID (e.g., "40012")
    pub plugin_id: String,

    /// Vulnerability name (e.g., "Cross Site Scripting (Reflected)")
    pub plugin_name: String,

    /// Severity level: High, Medium, Low
    pub severity: String,

    /// CWE identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe_id: Option<String>,

    /// Detailed description (markdown)
    pub description: String,

    /// Vulnerability category (e.g., "Injection")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Reference URLs for the vulnerability
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,

    /// OWASP cheatsheet URL for remediation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cheatsheet: Option<String>,

    /// Remediation advice from org findings report (most valuable for AI agents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_advice: Option<String>,

    /// Total number of affected paths
    pub total_paths: usize,

    /// Status breakdown for this finding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_summary: Option<StatusCounts>,

    /// All affected paths with full evidence and HTTP messages
    pub paths: Vec<PathFull>,
}

/// Full detail for a single affected path/endpoint
#[derive(Debug, Clone, Serialize)]
pub struct PathFull {
    /// URI ID (for future triage operations)
    pub uri_id: String,

    /// Stable SHA-256 hash identifying this finding across scans
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding_hash: Option<String>,

    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Affected URI path
    pub uri: String,

    /// Triage status
    pub status: String,

    /// Triage note/comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triage_note: Option<String>,

    /// Evidence found (e.g., error message snippet)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,

    /// Vulnerable parameter name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,

    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_info: Option<String>,

    /// Curl command to reproduce the finding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_command: Option<String>,

    /// ISO 8601 timestamp of first detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,

    /// ISO 8601 timestamp of last detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<String>,

    /// HTTP request details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<HttpMessage>,

    /// HTTP response details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<HttpMessage>,
}

/// HTTP request or response message
#[derive(Debug, Clone, Serialize)]
pub struct HttpMessage {
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<String>,

    /// Body content (truncated if exceeding max_body_size)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Whether the body was truncated
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub truncated: bool,
}

/// Output metadata for diagnostics and tooling
#[derive(Debug, Clone, Serialize)]
pub struct OutputMeta {
    /// ISO 8601 timestamp of when this output was generated
    pub generated_at: String,

    /// HawkOp CLI version
    pub hawkop_version: String,

    /// Number of API calls made to produce this output
    pub api_calls_made: usize,

    /// Time spent fetching data (milliseconds)
    pub fetch_duration_ms: u64,

    /// Whether any findings were omitted (due to --max-findings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings_omitted: Option<usize>,

    /// Whether any response bodies were truncated (due to --max-body-size)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub bodies_truncated: bool,
}

/// Helper for skip_serializing_if on usize fields
fn is_zero(v: &usize) -> bool {
    *v == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_full_detail_serializes() {
        let detail = ScanFullDetail {
            schema_version: "1.0".to_string(),
            scan: ScanInfo {
                id: "scan-123".to_string(),
                application_id: "app-456".to_string(),
                application_name: "Test App".to_string(),
                environment: "Development".to_string(),
                host: Some("https://example.com".to_string()),
                status: "COMPLETED".to_string(),
                completed_at: Some("2026-03-28T10:30:00Z".to_string()),
                duration_seconds: Some(120.0),
                hawkscan_version: "5.2.0".to_string(),
                policy: Some("OpenAPI/REST API".to_string()),
                user: Some("alice@example.com".to_string()),
                tags: HashMap::new(),
            },
            summary: FindingsSummary {
                total_findings: 5,
                by_severity: SeverityCounts {
                    high: 2,
                    medium: 2,
                    low: 1,
                    informational: 0,
                },
                by_status: StatusCounts {
                    new: 4,
                    assigned: 1,
                    accepted: 0,
                    false_positive: 0,
                },
                urls_scanned: Some(150),
            },
            findings: vec![FindingFull {
                plugin_id: "40012".to_string(),
                plugin_name: "Cross Site Scripting (Reflected)".to_string(),
                severity: "High".to_string(),
                cwe_id: Some("79".to_string()),
                description: "Reflected XSS vulnerability".to_string(),
                category: Some("Injection".to_string()),
                references: vec!["https://owasp.org/xss".to_string()],
                cheatsheet: None,
                remediation_advice: Some("Encode output, use CSP".to_string()),
                total_paths: 1,
                status_summary: None,
                paths: vec![PathFull {
                    uri_id: "uri-1".to_string(),
                    finding_hash: Some("abc123".to_string()),
                    method: "GET".to_string(),
                    uri: "/api/search?q=<payload>".to_string(),
                    status: "NEW".to_string(),
                    triage_note: None,
                    evidence: Some("Script tag reflected".to_string()),
                    param: Some("q".to_string()),
                    other_info: None,
                    validation_command: Some("curl -s 'https://...'".to_string()),
                    first_seen: Some("2026-03-01T00:00:00Z".to_string()),
                    last_seen: Some("2026-03-28T10:30:00Z".to_string()),
                    request: Some(HttpMessage {
                        headers: Some("GET /api/search?q=test HTTP/1.1".to_string()),
                        body: None,
                        truncated: false,
                    }),
                    response: Some(HttpMessage {
                        headers: Some("HTTP/1.1 200 OK".to_string()),
                        body: Some("<html>reflected</html>".to_string()),
                        truncated: false,
                    }),
                }],
            }],
            meta: OutputMeta {
                generated_at: "2026-03-28T10:32:00Z".to_string(),
                hawkop_version: "0.5.0".to_string(),
                api_calls_made: 15,
                fetch_duration_ms: 2340,
                findings_omitted: None,
                bodies_truncated: false,
            },
        };

        let json = serde_json::to_string_pretty(&detail).unwrap();
        assert!(json.contains("\"schema_version\": \"1.0\""));
        assert!(json.contains("\"plugin_id\": \"40012\""));
        assert!(json.contains("\"remediation_advice\""));
        assert!(json.contains("\"validation_command\""));
        assert!(json.contains("\"finding_hash\": \"abc123\""));
    }

    #[test]
    fn test_severity_counts_skip_zero() {
        let counts = SeverityCounts {
            high: 1,
            medium: 0,
            low: 0,
            informational: 0,
        };
        let json = serde_json::to_string(&counts).unwrap();
        assert!(json.contains("\"high\":1"));
        // informational should be omitted when zero
        assert!(!json.contains("informational"));
    }

    #[test]
    fn test_http_message_truncated_skipped_when_false() {
        let msg = HttpMessage {
            headers: Some("GET / HTTP/1.1".to_string()),
            body: None,
            truncated: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("truncated"));
    }

    #[test]
    fn test_http_message_truncated_shown_when_true() {
        let msg = HttpMessage {
            headers: Some("GET / HTTP/1.1".to_string()),
            body: Some("...".to_string()),
            truncated: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"truncated\":true"));
    }
}
