//! Scan configuration display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::ScanConfig;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_display_from_scan_config() {
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
        let config = ScanConfig {
            name: "minimal-config".to_string(),
            description: None,
            organization_id: None,
        };

        let display = ConfigDisplay::from(config);

        assert_eq!(display.name, "minimal-config");
        assert_eq!(display.description, "--");
    }
}
