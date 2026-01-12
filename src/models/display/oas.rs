//! OpenAPI specification display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::OASAsset;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oas_display_from_oas_asset() {
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
}
