//! Policy display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::{OrgPolicy, PolicyType, StackHawkPolicy};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_display_from_stackhawk_policy() {
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
}
