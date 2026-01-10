//! Organization display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::Organization;

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
}
