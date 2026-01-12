//! Application models

use serde::{Deserialize, Serialize};

/// Application resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// Application ID
    #[serde(rename = "applicationId")]
    pub id: String,

    /// Application name
    pub name: String,

    /// Environment name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,

    /// Risk level (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "riskLevel")]
    pub risk_level: Option<String>,

    /// Application status (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "applicationStatus")]
    pub status: Option<String>,

    /// Organization ID (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "organizationId")]
    pub organization_id: Option<String>,

    /// Application type: "STANDARD" or "CLOUD"
    #[serde(skip_serializing_if = "Option::is_none", rename = "applicationType")]
    pub application_type: Option<String>,

    /// Cloud scan target (only for CLOUD apps)
    #[serde(skip_serializing_if = "Option::is_none", rename = "cloudScanTarget")]
    pub cloud_scan_target: Option<CloudScanTarget>,
}

/// Cloud scan target for hosted/cloud applications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudScanTarget {
    /// Target URL to scan
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,

    /// Whether the domain has been verified
    #[serde(default)]
    pub is_domain_verified: bool,
}
