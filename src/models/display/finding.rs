//! Alert and finding display models

use serde::Serialize;
use tabled::Tabled;

use super::common::truncate_string;
use crate::client::models::{
    AlertMsgResponse, AlertResponse, ApplicationAlert, ApplicationAlertUri,
};

/// Alert (plugin) display model for `scan <id> alerts` table.
///
/// Note: Replaced by `PrettyAlertDisplay` for the default view, but kept for
/// backwards compatibility and potential future use in table format.
#[allow(dead_code)]
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct AlertDisplay {
    /// Plugin ID
    #[tabled(rename = "PLUGIN")]
    pub plugin_id: String,

    /// Severity level (High, Medium, Low)
    #[tabled(rename = "SEVERITY")]
    pub severity: String,

    /// Plugin/vulnerability name
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Number of affected paths
    #[tabled(rename = "PATHS")]
    pub path_count: String,

    /// New (untriaged) findings count
    #[tabled(rename = "NEW")]
    pub new_count: String,

    /// Triaged findings count
    #[tabled(rename = "TRIAGED")]
    pub triaged_count: String,

    /// CWE identifier
    #[tabled(rename = "CWE")]
    pub cwe: String,
}

impl From<ApplicationAlert> for AlertDisplay {
    fn from(alert: ApplicationAlert) -> Self {
        // Count new (UNKNOWN) vs triaged (PROMOTED, etc.) findings
        let mut new_count = 0u32;
        let mut triaged_count = 0u32;

        for status_stat in &alert.alert_status_stats {
            let is_new = status_stat.alert_status == "UNKNOWN";
            let is_triaged = status_stat.alert_status == "PROMOTED"
                || status_stat.alert_status == "ACCEPTED"
                || status_stat.alert_status == "FALSE_POSITIVE"
                || status_stat.alert_status == "RISK_ACCEPTED";

            if is_new {
                new_count += status_stat.total_count;
            } else if is_triaged {
                triaged_count += status_stat.total_count;
            }
        }

        Self {
            plugin_id: alert.plugin_id,
            severity: alert.severity.clone(),
            name: truncate_string(&alert.name, 35),
            path_count: alert.uri_count.to_string(),
            new_count: new_count.to_string(),
            triaged_count: triaged_count.to_string(),
            cwe: alert
                .cwe_id
                .map(|c| format!("CWE-{}", c))
                .unwrap_or_else(|| "--".to_string()),
        }
    }
}

impl From<&ApplicationAlert> for AlertDisplay {
    fn from(alert: &ApplicationAlert) -> Self {
        AlertDisplay::from(alert.clone())
    }
}

/// Pretty alert display model for `scan get` with detailed triage columns.
///
/// This display format matches the mockup with columns:
/// PLUGIN | SEVERITY | NAME | PATHS | NEW | ASSIGNED | ACCEPTED | FALSE+ | CWE
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct PrettyAlertDisplay {
    /// Plugin ID
    #[tabled(rename = "PLUGIN")]
    pub plugin_id: String,

    /// Severity level (High, Medium, Low)
    #[tabled(rename = "SEVERITY")]
    pub severity: String,

    /// Plugin/vulnerability name
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Number of affected paths
    #[tabled(rename = "PATHS")]
    pub paths: String,

    /// New (UNKNOWN status) findings count
    #[tabled(rename = "NEW")]
    pub new: String,

    /// Assigned (PROMOTED status) findings count
    #[tabled(rename = "ASSIGNED")]
    pub assigned: String,

    /// Accepted (ACCEPTED/RISK_ACCEPTED status) findings count
    #[tabled(rename = "ACCEPTED")]
    pub accepted: String,

    /// False positive (FALSE_POSITIVE status) findings count
    #[tabled(rename = "FALSE+")]
    pub false_positive: String,

    /// CWE identifier
    #[tabled(rename = "CWE")]
    pub cwe: String,
}

impl From<ApplicationAlert> for PrettyAlertDisplay {
    fn from(alert: ApplicationAlert) -> Self {
        // Count by triage status
        let mut new_count = 0u32;
        let mut assigned_count = 0u32;
        let mut accepted_count = 0u32;
        let mut false_positive_count = 0u32;

        for status_stat in &alert.alert_status_stats {
            match status_stat.alert_status.as_str() {
                "UNKNOWN" => new_count += status_stat.total_count,
                "PROMOTED" => assigned_count += status_stat.total_count,
                "ACCEPTED" | "RISK_ACCEPTED" => accepted_count += status_stat.total_count,
                "FALSE_POSITIVE" => false_positive_count += status_stat.total_count,
                _ => {}
            }
        }

        Self {
            plugin_id: alert.plugin_id,
            severity: alert.severity.clone(),
            name: truncate_string(&alert.name, 25),
            paths: alert.uri_count.to_string(),
            new: new_count.to_string(),
            assigned: assigned_count.to_string(),
            accepted: accepted_count.to_string(),
            false_positive: false_positive_count.to_string(),
            cwe: alert
                .cwe_id
                .map(|c| format!("CWE-{}", c))
                .unwrap_or_else(|| "--".to_string()),
        }
    }
}

impl From<&ApplicationAlert> for PrettyAlertDisplay {
    fn from(alert: &ApplicationAlert) -> Self {
        PrettyAlertDisplay::from(alert.clone())
    }
}

/// Alert finding (path) display model for `scan <id> alert <plugin>` table.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct AlertFindingDisplay {
    /// HTTP method
    #[tabled(rename = "METHOD")]
    pub method: String,

    /// URI path
    #[tabled(rename = "PATH")]
    pub path: String,

    /// Triage status (New, Triaged, Accepted, False Positive)
    #[tabled(rename = "STATUS")]
    pub status: String,

    /// Alert URI ID (for drill-down)
    #[tabled(rename = "URI ID")]
    pub uri_id: String,

    /// Message ID (for drill-down)
    #[tabled(rename = "MSG")]
    pub msg_id: String,
}

impl From<ApplicationAlertUri> for AlertFindingDisplay {
    fn from(uri: ApplicationAlertUri) -> Self {
        Self {
            method: uri.request_method,
            path: truncate_string(&uri.uri, 50),
            status: format_triage_status(&uri.status),
            uri_id: uri.alert_uri_id,
            msg_id: uri.msg_id,
        }
    }
}

impl From<&ApplicationAlertUri> for AlertFindingDisplay {
    fn from(uri: &ApplicationAlertUri) -> Self {
        AlertFindingDisplay::from(uri.clone())
    }
}

/// Alert detail for multi-section display (`scan <id> alert <plugin>`)
#[derive(Debug, Clone, Serialize)]
pub struct AlertDetail {
    pub response: AlertResponse,
}

impl AlertDetail {
    pub fn new(response: AlertResponse) -> Self {
        Self { response }
    }

    /// Format header text (shown above the paths table)
    pub fn format_header(&self) -> String {
        let alert = &self.response.alert;
        let mut output = String::new();

        output.push_str(&format!(
            "{} ({}) - {}\n",
            alert.name, alert.plugin_id, alert.severity
        ));

        if let Some(ref cwe) = alert.cwe_id {
            output.push_str(&format!(
                "CWE-{} | {} paths affected\n",
                cwe, alert.uri_count
            ));
        } else {
            output.push_str(&format!("{} paths affected\n", alert.uri_count));
        }

        output
    }
}

/// Alert message display for HTTP request/response (`scan <id> alert <plugin> <uri> message`)
#[derive(Debug, Clone, Serialize)]
pub struct AlertMessageDetail {
    pub response: AlertMsgResponse,
    pub plugin_name: Option<String>,
    pub severity: Option<String>,
}

impl AlertMessageDetail {
    pub fn new(response: AlertMsgResponse) -> Self {
        Self {
            response,
            plugin_name: None,
            severity: None,
        }
    }

    pub fn with_context(mut self, plugin_name: &str, severity: &str) -> Self {
        self.plugin_name = Some(plugin_name.to_string());
        self.severity = Some(severity.to_string());
        self
    }

    /// Format as multi-section text output
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        let msg = &self.response;

        // Finding details header
        output.push_str("Finding Details\n");
        output.push_str("════════════════════════════════════════════════════════════════════\n");

        if let Some(ref name) = self.plugin_name {
            output.push_str(&format!("Plugin:   {}\n", name));
        }
        if let Some(ref severity) = self.severity {
            output.push_str(&format!("Severity: {}\n", severity));
        }
        output.push_str(&format!("URI:      {}\n", msg.uri));

        if let Some(ref evidence) = msg.evidence
            && !evidence.is_empty()
        {
            output.push_str(&format!("Evidence: {}\n", truncate_string(evidence, 60)));
        }
        if let Some(ref param) = msg.param
            && !param.is_empty()
        {
            output.push_str(&format!("Param:    {}\n", param));
        }

        // HTTP Request
        output.push_str("\n─── Request ───────────────────────────────────────────────────────\n");
        if let Some(ref headers) = msg.scan_message.request_header {
            output.push_str(headers);
            if !headers.ends_with('\n') {
                output.push('\n');
            }
        }
        if let Some(ref body) = msg.scan_message.request_body
            && !body.is_empty()
        {
            output.push('\n');
            output.push_str(&truncate_string(body, 500));
            output.push('\n');
        }

        // HTTP Response
        output.push_str("\n─── Response ──────────────────────────────────────────────────────\n");
        if let Some(ref headers) = msg.scan_message.response_header {
            output.push_str(headers);
            if !headers.ends_with('\n') {
                output.push('\n');
            }
        }
        if let Some(ref body) = msg.scan_message.response_body
            && !body.is_empty()
        {
            output.push('\n');
            output.push_str(&truncate_string(body, 2000));
            output.push('\n');
        }

        // Validation command
        if let Some(ref curl) = msg.validation_command
            && !curl.is_empty()
        {
            output.push_str(
                "\n─── Validation Command ────────────────────────────────────────────\n",
            );
            output.push_str(curl);
            output.push('\n');
        }

        output
    }
}

/// Format triage status for display
pub fn format_triage_status(status: &str) -> String {
    match status {
        "UNKNOWN" => "New".to_string(),
        "PROMOTED" => "Triaged".to_string(),
        "ACCEPTED" | "RISK_ACCEPTED" => "Accepted".to_string(),
        "FALSE_POSITIVE" => "False Pos".to_string(),
        other => other.to_string(),
    }
}
