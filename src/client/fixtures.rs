//! Test fixtures and builders for API model types
//!
//! Provides builder patterns for creating test data with sensible defaults.
//! Import via `use crate::client::fixtures::*` in test modules.

#![allow(dead_code)] // Builder methods are available for future tests

use std::collections::HashMap;

use super::models::{
    AlertStats, AlertStatusStats, Application, Organization, Scan, ScanResult, User, UserExternal,
};

// ============================================================================
// OrganizationBuilder
// ============================================================================

/// Builder for creating test Organization instances.
///
/// # Example
/// ```ignore
/// let org = OrganizationBuilder::new("org-123")
///     .name("Test Org")
///     .user_count(10)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct OrganizationBuilder {
    id: String,
    name: String,
    user_count: Option<usize>,
    app_count: Option<usize>,
}

impl OrganizationBuilder {
    /// Create a new builder with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: format!("Organization {}", &id),
            id,
            user_count: None,
            app_count: None,
        }
    }

    /// Set the organization name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the user count.
    pub fn user_count(mut self, count: usize) -> Self {
        self.user_count = Some(count);
        self
    }

    /// Set the application count.
    pub fn app_count(mut self, count: usize) -> Self {
        self.app_count = Some(count);
        self
    }

    /// Build the Organization.
    pub fn build(self) -> Organization {
        Organization {
            id: self.id,
            name: self.name,
            user_count: self.user_count,
            app_count: self.app_count,
        }
    }
}

// ============================================================================
// ApplicationBuilder
// ============================================================================

/// Builder for creating test Application instances.
///
/// # Example
/// ```ignore
/// let app = ApplicationBuilder::new("app-123")
///     .name("My App")
///     .env("production")
///     .org_id("org-456")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ApplicationBuilder {
    id: String,
    name: String,
    env: Option<String>,
    risk_level: Option<String>,
    status: Option<String>,
    organization_id: Option<String>,
    application_type: Option<String>,
}

impl ApplicationBuilder {
    /// Create a new builder with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: format!("Application {}", &id),
            id,
            env: None,
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: None,
        }
    }

    /// Set the application name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the environment.
    pub fn env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// Set the risk level.
    pub fn risk_level(mut self, level: impl Into<String>) -> Self {
        self.risk_level = Some(level.into());
        self
    }

    /// Set the status.
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Set the organization ID.
    pub fn org_id(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Set the application type.
    pub fn app_type(mut self, app_type: impl Into<String>) -> Self {
        self.application_type = Some(app_type.into());
        self
    }

    /// Build the Application.
    pub fn build(self) -> Application {
        Application {
            id: self.id,
            name: self.name,
            env: self.env,
            risk_level: self.risk_level,
            status: self.status,
            organization_id: self.organization_id,
            application_type: self.application_type,
            cloud_scan_target: None,
        }
    }
}

// ============================================================================
// ScanResultBuilder
// ============================================================================

/// Builder for creating test ScanResult instances.
///
/// # Example
/// ```ignore
/// let scan = ScanResultBuilder::new("scan-123", "app-456")
///     .status("COMPLETED")
///     .env("staging")
///     .with_findings(5, 3, 1)  // total new findings by severity
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ScanResultBuilder {
    scan_id: String,
    app_id: String,
    app_name: String,
    env: String,
    status: String,
    timestamp: String,
    scan_duration: Option<String>,
    hawkscan_version: Option<String>,
    alert_stats: Option<AlertStats>,
}

impl ScanResultBuilder {
    /// Create a new builder with the given scan ID and app ID.
    pub fn new(scan_id: impl Into<String>, app_id: impl Into<String>) -> Self {
        let scan_id = scan_id.into();
        let app_id = app_id.into();
        Self {
            app_name: format!("App {}", &app_id),
            scan_id,
            app_id,
            env: "development".to_string(),
            status: "COMPLETED".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis().to_string(),
            scan_duration: Some("120".to_string()),
            hawkscan_version: Some("3.0.0".to_string()),
            alert_stats: None,
        }
    }

    /// Set the application name.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// Set the environment.
    pub fn env(mut self, env: impl Into<String>) -> Self {
        self.env = env.into();
        self
    }

    /// Set the scan status (STARTED, COMPLETED, ERROR).
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set the timestamp (milliseconds since epoch as string).
    pub fn timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = ts.into();
        self
    }

    /// Set the duration in seconds.
    pub fn duration_seconds(mut self, secs: u64) -> Self {
        self.scan_duration = Some(secs.to_string());
        self
    }

    /// Set the HawkScan version.
    pub fn hawkscan_version(mut self, version: impl Into<String>) -> Self {
        self.hawkscan_version = Some(version.into());
        self
    }

    /// Add alert statistics with the given finding counts (new/UNKNOWN status).
    /// Arguments are: high, medium, low severity counts.
    pub fn with_findings(mut self, high: u32, medium: u32, low: u32) -> Self {
        let total = high + medium + low;
        let mut severity_stats = HashMap::new();
        if high > 0 {
            severity_stats.insert("High".to_string(), high);
        }
        if medium > 0 {
            severity_stats.insert("Medium".to_string(), medium);
        }
        if low > 0 {
            severity_stats.insert("Low".to_string(), low);
        }

        let stats = self.alert_stats.get_or_insert(AlertStats {
            total_alerts: 0,
            unique_alerts: 0,
            alert_status_stats: Vec::new(),
        });

        stats.total_alerts += total;
        stats.unique_alerts += total;
        stats.alert_status_stats.push(AlertStatusStats {
            alert_status: "UNKNOWN".to_string(),
            total_count: total,
            severity_stats,
        });

        self
    }

    /// Add triaged findings (PROMOTED status).
    pub fn with_triaged_findings(mut self, high: u32, medium: u32, low: u32) -> Self {
        let total = high + medium + low;
        let mut severity_stats = HashMap::new();
        if high > 0 {
            severity_stats.insert("High".to_string(), high);
        }
        if medium > 0 {
            severity_stats.insert("Medium".to_string(), medium);
        }
        if low > 0 {
            severity_stats.insert("Low".to_string(), low);
        }

        let stats = self.alert_stats.get_or_insert(AlertStats {
            total_alerts: 0,
            unique_alerts: 0,
            alert_status_stats: Vec::new(),
        });

        stats.total_alerts += total;
        stats.alert_status_stats.push(AlertStatusStats {
            alert_status: "PROMOTED".to_string(),
            total_count: total,
            severity_stats,
        });

        self
    }

    /// Build the ScanResult.
    pub fn build(self) -> ScanResult {
        ScanResult {
            scan: Scan {
                id: self.scan_id,
                application_id: self.app_id,
                application_name: self.app_name,
                env: self.env,
                status: self.status,
                timestamp: self.timestamp,
                version: self.hawkscan_version.unwrap_or_default(),
                external_user_id: None,
            },
            scan_duration: self.scan_duration,
            url_count: None,
            alert_stats: self.alert_stats,
            severity_stats: None,
            app_host: None,
            policy_name: None,
            tags: Vec::new(),
            metadata: None,
        }
    }
}

// ============================================================================
// UserBuilder
// ============================================================================

/// Builder for creating test User instances.
///
/// # Example
/// ```ignore
/// let user = UserBuilder::new("user-123")
///     .email("test@example.com")
///     .name("Test", "User")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct UserBuilder {
    id: String,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    full_name: Option<String>,
}

impl UserBuilder {
    /// Create a new builder with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            email: format!("{}@example.com", &id),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            full_name: None,
            id,
        }
    }

    /// Set the email address.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    /// Set the first and last name.
    pub fn name(mut self, first: impl Into<String>, last: impl Into<String>) -> Self {
        self.first_name = Some(first.into());
        self.last_name = Some(last.into());
        self
    }

    /// Set the full name.
    pub fn full_name(mut self, name: impl Into<String>) -> Self {
        self.full_name = Some(name.into());
        self
    }

    /// Build the User.
    pub fn build(self) -> User {
        User {
            external: UserExternal {
                id: self.id,
                email: self.email,
                first_name: self.first_name,
                last_name: self.last_name,
                full_name: self.full_name,
            },
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create a minimal test organization.
pub fn test_org(id: &str) -> Organization {
    OrganizationBuilder::new(id).build()
}

/// Create a minimal test application.
pub fn test_app(id: &str) -> Application {
    ApplicationBuilder::new(id).build()
}

/// Create a minimal test scan result.
pub fn test_scan(scan_id: &str, app_id: &str) -> ScanResult {
    ScanResultBuilder::new(scan_id, app_id).build()
}

/// Create a minimal test user.
pub fn test_user(id: &str) -> User {
    UserBuilder::new(id).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_builder_defaults() {
        let org = OrganizationBuilder::new("org-1").build();
        assert_eq!(org.id, "org-1");
        assert_eq!(org.name, "Organization org-1");
        assert!(org.user_count.is_none());
        assert!(org.app_count.is_none());
    }

    #[test]
    fn test_organization_builder_with_all_fields() {
        let org = OrganizationBuilder::new("org-2")
            .name("Custom Name")
            .user_count(10)
            .app_count(5)
            .build();

        assert_eq!(org.id, "org-2");
        assert_eq!(org.name, "Custom Name");
        assert_eq!(org.user_count, Some(10));
        assert_eq!(org.app_count, Some(5));
    }

    #[test]
    fn test_application_builder_defaults() {
        let app = ApplicationBuilder::new("app-1").build();
        assert_eq!(app.id, "app-1");
        assert_eq!(app.name, "Application app-1");
        assert!(app.env.is_none());
    }

    #[test]
    fn test_application_builder_with_env() {
        let app = ApplicationBuilder::new("app-2")
            .name("My App")
            .env("production")
            .org_id("org-1")
            .build();

        assert_eq!(app.id, "app-2");
        assert_eq!(app.name, "My App");
        assert_eq!(app.env, Some("production".to_string()));
        assert_eq!(app.organization_id, Some("org-1".to_string()));
    }

    #[test]
    fn test_scan_result_builder_defaults() {
        let scan = ScanResultBuilder::new("scan-1", "app-1").build();
        assert_eq!(scan.scan.id, "scan-1");
        assert_eq!(scan.scan.application_id, "app-1");
        assert_eq!(scan.scan.status, "COMPLETED");
        assert_eq!(scan.scan.env, "development");
    }

    #[test]
    fn test_scan_result_builder_with_findings() {
        let scan = ScanResultBuilder::new("scan-2", "app-2")
            .status("COMPLETED")
            .with_findings(2, 5, 10)
            .build();

        let stats = scan.alert_stats.expect("should have stats");
        assert_eq!(stats.total_alerts, 17); // 2 + 5 + 10
        assert_eq!(stats.alert_status_stats.len(), 1);

        let unknown_stats = &stats.alert_status_stats[0];
        assert_eq!(unknown_stats.alert_status, "UNKNOWN");
        assert_eq!(unknown_stats.total_count, 17);
        assert_eq!(unknown_stats.severity_stats.get("High"), Some(&2));
        assert_eq!(unknown_stats.severity_stats.get("Medium"), Some(&5));
        assert_eq!(unknown_stats.severity_stats.get("Low"), Some(&10));
    }

    #[test]
    fn test_scan_result_builder_with_triaged() {
        let scan = ScanResultBuilder::new("scan-3", "app-3")
            .with_findings(5, 0, 0)
            .with_triaged_findings(2, 0, 0)
            .build();

        let stats = scan.alert_stats.expect("should have stats");
        // Should have 2 entries: UNKNOWN + PROMOTED
        assert_eq!(stats.alert_status_stats.len(), 2);
        assert_eq!(stats.total_alerts, 7); // 5 new + 2 triaged
    }

    #[test]
    fn test_user_builder_defaults() {
        let user = UserBuilder::new("user-1").build();
        assert_eq!(user.external.id, "user-1");
        assert_eq!(user.external.email, "user-1@example.com");
        assert_eq!(user.external.first_name, Some("Test".to_string()));
        assert_eq!(user.external.last_name, Some("User".to_string()));
    }

    #[test]
    fn test_user_builder_custom() {
        let user = UserBuilder::new("user-2")
            .email("jane@company.com")
            .name("Jane", "Doe")
            .build();

        assert_eq!(user.external.email, "jane@company.com");
        assert_eq!(user.external.first_name, Some("Jane".to_string()));
        assert_eq!(user.external.last_name, Some("Doe".to_string()));
    }

    #[test]
    fn test_convenience_functions() {
        let org = test_org("quick-org");
        assert_eq!(org.id, "quick-org");

        let app = test_app("quick-app");
        assert_eq!(app.id, "quick-app");

        let scan = test_scan("quick-scan", "quick-app");
        assert_eq!(scan.scan.id, "quick-scan");

        let user = test_user("quick-user");
        assert_eq!(user.external.id, "quick-user");
    }
}
