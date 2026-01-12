//! Policy models

use serde::{Deserialize, Serialize};

/// Policy type (StackHawk preset or Organization custom)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// Preset policy created by StackHawk (read-only)
    StackHawk,
    /// Custom policy for an organization (editable)
    Organization,
}

impl std::fmt::Display for PolicyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyType::StackHawk => write!(f, "StackHawk"),
            PolicyType::Organization => write!(f, "Organization"),
        }
    }
}

/// StackHawk scan policy (preset, read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackHawkPolicy {
    /// Policy ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Policy name (unique identifier)
    pub name: String,

    /// Human-readable display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Policy description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Organization scan policy (custom, editable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgPolicy {
    /// Policy name (unique identifier)
    pub name: String,

    /// Human-readable display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Policy description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Organization ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}
