//! Audit log display model

use chrono::DateTime;
use serde::Serialize;
use tabled::Tabled;

use crate::client::models::AuditRecord;

/// Audit log display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct AuditDisplay {
    /// When the action occurred
    #[tabled(rename = "TIMESTAMP")]
    pub timestamp: String,

    /// Activity type (user or org type)
    #[tabled(rename = "TYPE")]
    pub activity_type: String,

    /// User who performed the action
    #[tabled(rename = "USER")]
    pub user: String,

    /// User email
    #[tabled(rename = "EMAIL")]
    pub email: String,

    /// Details extracted from payload
    #[tabled(rename = "DETAILS")]
    pub details: String,
}

impl From<AuditRecord> for AuditDisplay {
    fn from(record: AuditRecord) -> Self {
        // Format timestamp (API returns as string)
        let timestamp = if let Ok(ts) = record.timestamp.parse::<i64>() {
            format_audit_timestamp(ts)
        } else {
            "--".to_string()
        };

        // Use user activity type if present, otherwise org activity type
        let activity_type = record
            .user_activity_type
            .or(record.organization_activity_type)
            .unwrap_or_else(|| "--".to_string());

        // Parse payload JSON string and extract details
        let payload: serde_json::Value =
            serde_json::from_str(&record.payload).unwrap_or(serde_json::Value::Null);
        let details = extract_audit_details(&payload, &activity_type);

        Self {
            timestamp,
            activity_type,
            user: if record.user_name.is_empty() {
                "--".to_string()
            } else {
                record.user_name
            },
            email: if record.user_email.is_empty() {
                "--".to_string()
            } else {
                record.user_email
            },
            details,
        }
    }
}

impl From<&AuditRecord> for AuditDisplay {
    fn from(record: &AuditRecord) -> Self {
        AuditDisplay::from(record.clone())
    }
}

/// Format audit timestamp (milliseconds) to human-readable format
pub fn format_audit_timestamp(timestamp_ms: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp_millis(timestamp_ms) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else if let Some(dt) = DateTime::from_timestamp(timestamp_ms, 0) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "--".to_string()
    }
}

/// Extract relevant details from audit payload based on activity type
pub fn extract_audit_details(payload: &serde_json::Value, activity_type: &str) -> String {
    // Try to extract the most relevant field based on activity type
    let detail = match activity_type {
        // Application-related
        "APPLICATION_ADDED" | "APPLICATION_UPDATED" | "APPLICATION_DELETED" => payload
            .get("appName")
            .or_else(|| payload.get("applicationName"))
            .or_else(|| payload.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| format!("app: {}", s)),

        // Scan-related
        "SCAN_STARTED" | "SCAN_COMPLETED" | "SCAN_FAILED" => {
            let app = payload
                .get("appName")
                .or_else(|| payload.get("applicationName"))
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let env = payload
                .get("envName")
                .or_else(|| payload.get("env"))
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            Some(format!("{} ({})", app, env))
        }

        // User/Team-related
        "USER_ADDED" | "USER_REMOVED" | "USER_UPDATED" => payload
            .get("email")
            .or_else(|| payload.get("userName"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),

        "TEAM_CREATED" | "TEAM_UPDATED" | "TEAM_DELETED" => payload
            .get("teamName")
            .or_else(|| payload.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| format!("team: {}", s)),

        // Policy-related
        "POLICY_CREATED" | "POLICY_UPDATED" | "POLICY_DELETED" => payload
            .get("policyName")
            .or_else(|| payload.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| format!("policy: {}", s)),

        // API key related
        "API_KEY_CREATED" | "API_KEY_DELETED" => payload
            .get("keyName")
            .and_then(|v| v.as_str())
            .map(|s| format!("key: {}", s)),

        // External integrations
        "EXTERNAL_ALERTS_SENT" => payload
            .get("integration")
            .and_then(|v| v.as_str())
            .map(|s| format!("to {}", s)),

        // Default: try common fields
        _ => payload
            .get("appName")
            .or_else(|| payload.get("name"))
            .or_else(|| payload.get("applicationName"))
            .or_else(|| payload.get("message"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    // Truncate if too long
    match detail {
        Some(s) if s.len() > 40 => format!("{}...", &s[..37]),
        Some(s) => s,
        None => "--".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_display_from_audit_record() {
        let record = AuditRecord {
            id: "audit-123".to_string(),
            user_activity_type: Some("SCAN_STARTED".to_string()),
            organization_activity_type: None,
            organization_id: "org-456".to_string(),
            user_id: "user-789".to_string(),
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
            payload: r#"{"appName":"TestApp","envName":"Production"}"#.to_string(),
            timestamp: "1703721600000".to_string(),
            user_ip_addr: None,
        };

        let display = AuditDisplay::from(record);

        assert_eq!(display.activity_type, "SCAN_STARTED");
        assert_eq!(display.user, "John Doe");
        assert_eq!(display.email, "john@example.com");
        assert!(display.details.contains("TestApp"));
    }

    #[test]
    fn test_audit_display_org_activity() {
        let record = AuditRecord {
            id: "audit-456".to_string(),
            user_activity_type: None,
            organization_activity_type: Some("EXTERNAL_ALERTS_SENT".to_string()),
            organization_id: "org-789".to_string(),
            user_id: "".to_string(),
            user_name: "".to_string(),
            user_email: "".to_string(),
            payload: r#"{"integration":"JIRA"}"#.to_string(),
            timestamp: "1703721600000".to_string(),
            user_ip_addr: None,
        };

        let display = AuditDisplay::from(record);

        assert_eq!(display.activity_type, "EXTERNAL_ALERTS_SENT");
        assert_eq!(display.user, "--");
        assert_eq!(display.email, "--");
        assert!(display.details.contains("JIRA"));
    }
}
