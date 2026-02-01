//! OpenAPI specification models

use serde::{Deserialize, Serialize};

/// Hosted OpenAPI specification asset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OASAsset {
    /// Unique OAS ID
    #[serde(default)]
    pub oas_id: String,

    /// Repository ID this OAS belongs to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,

    /// Repository name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_name: Option<String>,

    /// Source root path in repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_root_path: Option<String>,

    /// File name of the OAS spec
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    /// File size in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

// ============================================================================
// OAS Mappings Response
// ============================================================================

/// Response from getting application-mapped OAS specs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationMappedOASResponse {
    /// The application ID
    #[serde(default)]
    pub app_id: String,

    /// OAS assets mapped to this application
    #[serde(default)]
    pub assets: Vec<OASAsset>,

    /// Repository IDs mapped to this application
    #[serde(default)]
    pub repo_ids: Vec<String>,
}
