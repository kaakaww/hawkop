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
}
