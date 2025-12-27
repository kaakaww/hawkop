//! Display model implementations for table and JSON output
//!
//! Display models transform API response types into CLI-friendly formats
//! with appropriate column names and serialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use tabled::Tabled;

use crate::client::{
    Application, OrgPolicy, Organization, PolicyType, Repository, ScanResult, StackHawkPolicy,
    Team, User, UserExternal,
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
    /// Application ID
    #[tabled(rename = "APP ID")]
    pub id: String,

    /// Application name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Application> for AppDisplay {
    fn from(app: Application) -> Self {
        Self {
            id: app.id,
            name: app.name,
        }
    }
}

impl From<&Application> for AppDisplay {
    fn from(app: &Application) -> Self {
        Self {
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

        // Format attack surface boolean (✓/✗)
        let attack_surface = if repo.is_in_attack_surface {
            "\u{2713}".to_string() // ✓
        } else {
            "\u{2717}".to_string() // ✗
        };

        // Format OAS boolean (✓/✗)
        let oas = if repo.has_generated_open_api_spec {
            "\u{2713}".to_string() // ✓
        } else {
            "\u{2717}".to_string() // ✗
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
        };

        let display = AppDisplay::from(app);

        assert_eq!(display.id, "app-789");
        assert_eq!(display.name, "Test App");
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
        };

        let display = AppDisplay::from(&app);

        assert_eq!(display.id, "app-abc");
        assert_eq!(display.name, "Another App");
    }
}
