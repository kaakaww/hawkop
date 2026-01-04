//! Display model implementations for table and JSON output
//!
//! Display models transform API response types into CLI-friendly formats
//! with appropriate column names and serialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use tabled::Tabled;

use crate::client::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, ApplicationAlertUri,
    AuditRecord, OASAsset, OrgPolicy, Organization, PolicyType, Repository, ScanConfig, ScanResult,
    Secret, StackHawkPolicy, Team, User, UserExternal,
};

/// Organization display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct OrgDisplay {
    /// Organization ID
    #[tabled(rename = "ORG ID")]
    pub id: String,

    /// Organization name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Organization> for OrgDisplay {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id,
            name: org.name,
        }
    }
}

impl From<&Organization> for OrgDisplay {
    fn from(org: &Organization) -> Self {
        Self {
            id: org.id.clone(),
            name: org.name.clone(),
        }
    }
}

/// Application display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct AppDisplay {
    /// Whether this is a cloud/hosted app
    #[tabled(rename = "CLOUD")]
    pub cloud: String,

    /// Application ID
    #[tabled(rename = "APP ID")]
    pub id: String,

    /// Application name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Application> for AppDisplay {
    fn from(app: Application) -> Self {
        let is_cloud = app
            .application_type
            .as_ref()
            .map(|t| t == "CLOUD")
            .unwrap_or(false);

        Self {
            cloud: if is_cloud {
                "\u{2713}".to_string() // ✓
            } else {
                "".to_string()
            },
            id: app.id,
            name: app.name,
        }
    }
}

impl From<&Application> for AppDisplay {
    fn from(app: &Application) -> Self {
        let is_cloud = app
            .application_type
            .as_ref()
            .map(|t| t == "CLOUD")
            .unwrap_or(false);

        Self {
            cloud: if is_cloud {
                "\u{2713}".to_string() // ✓
            } else {
                "".to_string()
            },
            id: app.id.clone(),
            name: app.name.clone(),
        }
    }
}

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

/// User/member display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct UserDisplay {
    /// User ID
    #[tabled(rename = "USER ID")]
    pub id: String,

    /// User email
    #[tabled(rename = "EMAIL")]
    pub email: String,

    /// User name (first + last)
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Role in organization
    #[tabled(rename = "ROLE")]
    pub role: String,
}

impl From<User> for UserDisplay {
    fn from(user: User) -> Self {
        UserDisplay::from(user.external)
    }
}

impl From<&User> for UserDisplay {
    fn from(user: &User) -> Self {
        UserDisplay::from(user.external.clone())
    }
}

impl From<UserExternal> for UserDisplay {
    fn from(user: UserExternal) -> Self {
        // Prefer full_name if available, otherwise combine first/last
        let name = user
            .full_name
            .unwrap_or_else(|| match (&user.first_name, &user.last_name) {
                (Some(first), Some(last)) => format!("{} {}", first, last),
                (Some(first), None) => first.clone(),
                (None, Some(last)) => last.clone(),
                (None, None) => "--".to_string(),
            });

        Self {
            id: user.id,
            email: user.email,
            name,
            role: "--".to_string(), // Role not available in current API response
        }
    }
}

/// Team display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct TeamDisplay {
    /// Team ID
    #[tabled(rename = "TEAM ID")]
    pub id: String,

    /// Team name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Team> for TeamDisplay {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
        }
    }
}

impl From<&Team> for TeamDisplay {
    fn from(team: &Team) -> Self {
        TeamDisplay::from(team.clone())
    }
}

/// Policy display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct PolicyDisplay {
    /// Policy type (StackHawk or Organization)
    #[tabled(rename = "TYPE")]
    pub policy_type: String,

    /// Human-readable display name
    #[tabled(rename = "DISPLAY NAME")]
    pub display_name: String,

    /// Policy name (identifier)
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Policy description
    #[tabled(rename = "DESCRIPTION")]
    pub description: String,
}

impl PolicyDisplay {
    /// Create from a StackHawk policy
    pub fn from_stackhawk(policy: StackHawkPolicy) -> Self {
        Self {
            policy_type: PolicyType::StackHawk.to_string(),
            display_name: policy.display_name.unwrap_or_else(|| "--".to_string()),
            name: policy.name,
            description: policy.description.unwrap_or_else(|| "--".to_string()),
        }
    }

    /// Create from an organization policy
    pub fn from_org(policy: OrgPolicy) -> Self {
        Self {
            policy_type: PolicyType::Organization.to_string(),
            display_name: policy.display_name.unwrap_or_else(|| "--".to_string()),
            name: policy.name,
            description: policy.description.unwrap_or_else(|| "--".to_string()),
        }
    }
}

/// Repository display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct RepoDisplay {
    /// Whether repo is in attack surface
    #[tabled(rename = "SURFACE")]
    pub attack_surface: String,

    /// Git provider (GITHUB, AZURE_DEVOPS, BITBUCKET, GITLAB)
    #[tabled(rename = "SRC")]
    pub provider: String,

    /// Git organization name
    #[tabled(rename = "ORG")]
    pub git_org: String,

    /// Repository name
    #[tabled(rename = "REPO")]
    pub name: String,

    /// Whether StackHawk generated an OAS
    #[tabled(rename = "OAS")]
    pub oas: String,

    /// Sensitive data tags
    #[tabled(rename = "SENSITIVE")]
    pub sensitive_data: String,

    /// Last commit time (ISO datetime)
    #[tabled(rename = "COMMITTED")]
    pub last_commit: String,

    /// Last committer
    #[tabled(rename = "BY")]
    pub last_committer: String,

    /// 30-day commit activity
    #[tabled(rename = "30D")]
    pub commit_count: String,

    /// Number of mapped apps
    #[tabled(rename = "APPS")]
    pub app_count: String,
}

impl From<Repository> for RepoDisplay {
    fn from(repo: Repository) -> Self {
        // Format provider
        let provider = repo.repo_source.unwrap_or_else(|| "--".to_string());

        // Format git org
        let git_org = repo.provider_org_name.unwrap_or_else(|| "--".to_string());

        // Format attack surface boolean (✓ or empty)
        let attack_surface = if repo.is_in_attack_surface {
            "\u{2713}".to_string() // ✓
        } else {
            "".to_string()
        };

        // Format OAS boolean (✓ or empty)
        let oas = if repo.has_generated_open_api_spec {
            "\u{2713}".to_string() // ✓
        } else {
            "".to_string()
        };

        // Format sensitive data tags (comma-separated, truncated)
        let sensitive_data = if repo.sensitive_data_tags.is_empty() {
            "--".to_string()
        } else {
            let tags: Vec<String> = repo
                .sensitive_data_tags
                .iter()
                .map(|t| t.name.clone())
                .collect();
            let joined = tags.join(", ");
            if joined.len() > 20 {
                format!("{}...", &joined[..17])
            } else {
                joined
            }
        };

        // Format last commit time as ISO datetime
        let last_commit = repo
            .last_commit_timestamp
            .as_ref()
            .map(|ts| format_as_iso_datetime(ts))
            .unwrap_or_else(|| "--".to_string());

        // Format last committer
        let last_committer = repo
            .last_contributor
            .and_then(|c| c.name)
            .unwrap_or_else(|| "--".to_string());

        // Format commit count
        let commit_count = repo.commit_count.to_string();

        // Format app count
        let app_count = repo.app_infos.len().to_string();

        Self {
            attack_surface,
            provider,
            git_org,
            name: repo.name,
            oas,
            sensitive_data,
            last_commit,
            last_committer,
            commit_count,
            app_count,
        }
    }
}

impl From<&Repository> for RepoDisplay {
    fn from(repo: &Repository) -> Self {
        RepoDisplay::from(repo.clone())
    }
}

/// OpenAPI specification asset display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct OASDisplay {
    /// OAS ID
    #[tabled(rename = "OAS ID")]
    pub id: String,

    /// Repository name
    #[tabled(rename = "REPO")]
    pub repo: String,

    /// Source root path
    #[tabled(rename = "PATH")]
    pub path: String,
}

impl From<OASAsset> for OASDisplay {
    fn from(oas: OASAsset) -> Self {
        Self {
            id: oas.oas_id,
            repo: oas.repository_name.unwrap_or_else(|| "--".to_string()),
            path: oas.source_root_path.unwrap_or_else(|| "--".to_string()),
        }
    }
}

impl From<&OASAsset> for OASDisplay {
    fn from(oas: &OASAsset) -> Self {
        OASDisplay::from(oas.clone())
    }
}

/// Scan configuration display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct ConfigDisplay {
    /// Configuration name
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Configuration description
    #[tabled(rename = "DESCRIPTION")]
    pub description: String,
}

impl From<ScanConfig> for ConfigDisplay {
    fn from(config: ScanConfig) -> Self {
        let description = config
            .description
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "--".to_string());

        Self {
            name: config.name,
            description,
        }
    }
}

impl From<&ScanConfig> for ConfigDisplay {
    fn from(config: &ScanConfig) -> Self {
        ConfigDisplay::from(config.clone())
    }
}

/// Secret display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct SecretDisplay {
    /// Secret name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Secret> for SecretDisplay {
    fn from(secret: Secret) -> Self {
        Self { name: secret.name }
    }
}

impl From<&Secret> for SecretDisplay {
    fn from(secret: &Secret) -> Self {
        SecretDisplay::from(secret.clone())
    }
}

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
fn format_audit_timestamp(timestamp_ms: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp_millis(timestamp_ms) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else if let Some(dt) = DateTime::from_timestamp(timestamp_ms, 0) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "--".to_string()
    }
}

// ============================================================================
// Scan Drill-Down Display Models
// ============================================================================

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

/// Count findings by severity from AlertStats
///
/// Note: Used by ScanOverview which is currently not in use but kept for
/// backwards compatibility.
#[allow(dead_code)]
fn count_findings_by_severity(stats: &crate::client::AlertStats) -> (u32, u32, u32, u32, u32, u32) {
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

/// Format triage status for display
fn format_triage_status(status: &str) -> String {
    match status {
        "UNKNOWN" => "New".to_string(),
        "PROMOTED" => "Triaged".to_string(),
        "ACCEPTED" | "RISK_ACCEPTED" => "Accepted".to_string(),
        "FALSE_POSITIVE" => "False Pos".to_string(),
        other => other.to_string(),
    }
}

/// Truncate string to max length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Extract relevant details from audit payload based on activity type
fn extract_audit_details(payload: &serde_json::Value, activity_type: &str) -> String {
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

/// Format timestamp string to ISO datetime (YYYY-MM-DDTHH:MM:SSZ)
fn format_as_iso_datetime(timestamp: &str) -> String {
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

/// Format scan status for display (normalize case)
fn format_status(status: &str) -> String {
    match status.to_uppercase().as_str() {
        "STARTED" => "Running".to_string(),
        "COMPLETED" => "Complete".to_string(),
        "ERROR" => "Failed".to_string(),
        other => other.to_string(),
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(seconds: f64) -> String {
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
fn format_relative_time(timestamp_ms: i64) -> String {
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
fn format_findings(result: &ScanResult) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_display_from_organization() {
        let org = Organization {
            id: "org-123".to_string(),
            name: "Test Org".to_string(),
            user_count: Some(10),
            app_count: Some(5),
        };

        let display = OrgDisplay::from(org);

        assert_eq!(display.id, "org-123");
        assert_eq!(display.name, "Test Org");
    }

    #[test]
    fn test_org_display_from_ref() {
        let org = Organization {
            id: "org-456".to_string(),
            name: "Another Org".to_string(),
            user_count: None,
            app_count: None,
        };

        let display = OrgDisplay::from(&org);

        assert_eq!(display.id, "org-456");
        assert_eq!(display.name, "Another Org");
    }

    #[test]
    fn test_app_display_from_application() {
        let app = Application {
            id: "app-789".to_string(),
            name: "Test App".to_string(),
            env: Some("production".to_string()),
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: None,
            cloud_scan_target: None,
        };

        let display = AppDisplay::from(app);

        assert_eq!(display.id, "app-789");
        assert_eq!(display.name, "Test App");
        assert_eq!(display.cloud, ""); // Not a cloud app
    }

    #[test]
    fn test_app_display_from_ref() {
        let app = Application {
            id: "app-abc".to_string(),
            name: "Another App".to_string(),
            env: None,
            risk_level: Some("High".to_string()),
            status: Some("Active".to_string()),
            organization_id: Some("org-123".to_string()),
            application_type: None,
            cloud_scan_target: None,
        };

        let display = AppDisplay::from(&app);

        assert_eq!(display.id, "app-abc");
        assert_eq!(display.name, "Another App");
        assert_eq!(display.cloud, ""); // Not a cloud app
    }

    #[test]
    fn test_app_display_cloud_app() {
        use crate::client::CloudScanTarget;

        let app = Application {
            id: "cloud-app-123".to_string(),
            name: "Cloud App".to_string(),
            env: None,
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: Some("CLOUD".to_string()),
            cloud_scan_target: Some(CloudScanTarget {
                target_url: Some("https://example.com".to_string()),
                is_domain_verified: true,
            }),
        };

        let display = AppDisplay::from(app);

        assert_eq!(display.id, "cloud-app-123");
        assert_eq!(display.name, "Cloud App");
        assert_eq!(display.cloud, "\u{2713}"); // ✓ for cloud apps
    }

    #[test]
    fn test_scan_display_from_scan_result() {
        use crate::client::{Scan, ScanResult};

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
        use crate::client::{AlertStats, AlertStatusStats, Scan, ScanResult};
        use std::collections::HashMap;

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
    fn test_team_display_from_team() {
        use crate::client::Team;

        let team = Team {
            id: "team-123".to_string(),
            name: "Security Team".to_string(),
            organization_id: Some("org-456".to_string()),
        };

        let display = TeamDisplay::from(team);

        assert_eq!(display.id, "team-123");
        assert_eq!(display.name, "Security Team");
    }

    #[test]
    fn test_user_display_from_user() {
        use crate::client::{User, UserExternal};

        let user = User {
            external: UserExternal {
                id: "user-123".to_string(),
                email: "test@example.com".to_string(),
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                full_name: Some("John Doe".to_string()),
            },
        };

        let display = UserDisplay::from(user);

        assert_eq!(display.id, "user-123");
        assert_eq!(display.email, "test@example.com");
        assert_eq!(display.name, "John Doe");
    }

    #[test]
    fn test_user_display_without_full_name() {
        use crate::client::{User, UserExternal};

        let user = User {
            external: UserExternal {
                id: "user-456".to_string(),
                email: "jane@example.com".to_string(),
                first_name: Some("Jane".to_string()),
                last_name: Some("Smith".to_string()),
                full_name: None,
            },
        };

        let display = UserDisplay::from(user);

        assert_eq!(display.id, "user-456");
        assert_eq!(display.email, "jane@example.com");
        assert_eq!(display.name, "Jane Smith");
    }

    #[test]
    fn test_policy_display_from_stackhawk_policy() {
        use crate::client::{PolicyType, StackHawkPolicy};

        let policy = StackHawkPolicy {
            id: Some("policy-123".to_string()),
            name: "api-scan".to_string(),
            display_name: Some("API Scan Policy".to_string()),
            description: Some("Default API scanning policy".to_string()),
        };

        let display = PolicyDisplay::from_stackhawk(policy);

        assert_eq!(display.name, "api-scan");
        assert_eq!(display.display_name, "API Scan Policy");
        assert_eq!(display.policy_type, PolicyType::StackHawk.to_string());
    }

    #[test]
    fn test_policy_display_from_org_policy() {
        use crate::client::{OrgPolicy, PolicyType};

        let policy = OrgPolicy {
            name: "custom-policy".to_string(),
            display_name: Some("Custom Scan Policy".to_string()),
            description: Some("Organization custom policy".to_string()),
            organization_id: Some("org-123".to_string()),
        };

        let display = PolicyDisplay::from_org(policy);

        assert_eq!(display.name, "custom-policy");
        assert_eq!(display.display_name, "Custom Scan Policy");
        assert_eq!(display.policy_type, PolicyType::Organization.to_string());
    }

    #[test]
    fn test_repo_display_from_repository() {
        use crate::client::Repository;

        let repo = Repository {
            id: Some("repo-123".to_string()),
            repo_source: Some("GITHUB".to_string()),
            provider_org_name: Some("myorg".to_string()),
            name: "my-api".to_string(),
            open_api_spec_info: None,
            has_generated_open_api_spec: true,
            is_in_attack_surface: true,
            framework_names: vec!["Spring".to_string()],
            sensitive_data_tags: vec![],
            last_commit_timestamp: None,
            last_contributor: None,
            commit_count: 10,
            app_infos: vec![],
            insights: vec![],
        };

        let display = RepoDisplay::from(repo);

        assert_eq!(display.name, "my-api");
        assert_eq!(display.provider, "GITHUB");
        assert_eq!(display.attack_surface, "\u{2713}"); // ✓
        assert_eq!(display.oas, "\u{2713}"); // ✓
    }

    #[test]
    fn test_repo_display_not_in_attack_surface() {
        use crate::client::Repository;

        let repo = Repository {
            id: Some("repo-456".to_string()),
            repo_source: Some("GITLAB".to_string()),
            provider_org_name: None,
            name: "internal-tool".to_string(),
            open_api_spec_info: None,
            has_generated_open_api_spec: false,
            is_in_attack_surface: false,
            framework_names: vec![],
            sensitive_data_tags: vec![],
            last_commit_timestamp: None,
            last_contributor: None,
            commit_count: 0,
            app_infos: vec![],
            insights: vec![],
        };

        let display = RepoDisplay::from(repo);

        assert_eq!(display.name, "internal-tool");
        assert_eq!(display.attack_surface, ""); // Empty for false
        assert_eq!(display.oas, ""); // Empty for false
    }

    #[test]
    fn test_oas_display_from_oas_asset() {
        use crate::client::OASAsset;

        let oas = OASAsset {
            oas_id: "oas-123".to_string(),
            repository_id: Some("repo-456".to_string()),
            repository_name: Some("my-api".to_string()),
            source_root_path: Some("/src/main".to_string()),
        };

        let display = OASDisplay::from(oas);

        assert_eq!(display.id, "oas-123");
        assert_eq!(display.repo, "my-api");
        assert_eq!(display.path, "/src/main");
    }

    #[test]
    fn test_oas_display_without_repo() {
        use crate::client::OASAsset;

        let oas = OASAsset {
            oas_id: "oas-789".to_string(),
            repository_id: None,
            repository_name: None,
            source_root_path: None,
        };

        let display = OASDisplay::from(oas);

        assert_eq!(display.id, "oas-789");
        assert_eq!(display.repo, "--");
        assert_eq!(display.path, "--");
    }

    #[test]
    fn test_config_display_from_scan_config() {
        use crate::client::ScanConfig;

        let config = ScanConfig {
            name: "prod-config".to_string(),
            description: Some("Production scan configuration".to_string()),
            organization_id: Some("org-123".to_string()),
        };

        let display = ConfigDisplay::from(config);

        assert_eq!(display.name, "prod-config");
        assert_eq!(display.description, "Production scan configuration");
    }

    #[test]
    fn test_config_display_without_description() {
        use crate::client::ScanConfig;

        let config = ScanConfig {
            name: "minimal-config".to_string(),
            description: None,
            organization_id: None,
        };

        let display = ConfigDisplay::from(config);

        assert_eq!(display.name, "minimal-config");
        assert_eq!(display.description, "--");
    }

    #[test]
    fn test_secret_display_from_secret() {
        use crate::client::Secret;

        let secret = Secret {
            name: "API_KEY".to_string(),
        };

        let display = SecretDisplay::from(secret);

        assert_eq!(display.name, "API_KEY");
    }

    #[test]
    fn test_audit_display_from_audit_record() {
        use crate::client::AuditRecord;

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
        use crate::client::AuditRecord;

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
        use crate::client::{Scan, ScanResult};

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
        use crate::client::{AlertStats, AlertStatusStats, Scan, ScanResult};
        use std::collections::HashMap;

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
