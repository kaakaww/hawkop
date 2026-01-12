//! Application display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::Application;

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
                "\u{2713}".to_string() // checkmark
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
                "\u{2713}".to_string() // checkmark
            } else {
                "".to_string()
            },
            id: app.id.clone(),
            name: app.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::models::CloudScanTarget;

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
        assert_eq!(display.cloud, "\u{2713}"); // checkmark for cloud apps
    }
}
