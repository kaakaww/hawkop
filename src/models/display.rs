//! Display model implementations for table and JSON output
//!
//! Display models transform API response types into CLI-friendly formats
//! with appropriate column names and serialization.

use serde::Serialize;
use tabled::Tabled;

use crate::client::{Application, Organization};

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
