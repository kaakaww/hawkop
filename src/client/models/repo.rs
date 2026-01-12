//! Repository models for attack surface mapping

use serde::{Deserialize, Serialize};

/// Repository from attack surface mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// Repository ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Git provider (GITHUB, AZURE_DEVOPS, BITBUCKET, GITLAB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_source: Option<String>,

    /// Provider organization name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_org_name: Option<String>,

    /// Repository name
    #[serde(default)]
    pub name: String,

    /// OpenAPI spec information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_api_spec_info: Option<OpenApiSpecInfo>,

    /// Whether StackHawk has generated an OpenAPI spec
    #[serde(default)]
    pub has_generated_open_api_spec: bool,

    /// Whether this repo is in the attack surface
    #[serde(default)]
    pub is_in_attack_surface: bool,

    /// Detected framework names
    #[serde(default)]
    pub framework_names: Vec<String>,

    /// Sensitive data tags detected
    #[serde(default)]
    pub sensitive_data_tags: Vec<SensitiveDataTag>,

    /// Timestamp of last commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_timestamp: Option<String>,

    /// Last contributor information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_contributor: Option<RepoContributor>,

    /// Commit count (30-day activity)
    #[serde(default)]
    pub commit_count: u32,

    /// Mapped application info
    #[serde(default)]
    pub app_infos: Vec<RepoAppInfo>,

    /// API Discovery insights
    #[serde(default)]
    pub insights: Vec<RepoInsight>,
}

/// OpenAPI specification info for repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiSpecInfo {
    /// Number of generated OAS files
    #[serde(default)]
    pub generated_oas_count: u32,
}

/// Sensitive data tag detected in repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataTag {
    /// Tag name (e.g., PII, PCI, PHI)
    #[serde(default)]
    pub name: String,
}

/// Repository contributor information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoContributor {
    /// Contributor name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Contributor email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Application info linked to repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoAppInfo {
    /// Application ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    /// Application name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
}

/// API Discovery insight for repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInsight {
    /// Insight name (e.g., "apiStyle")
    #[serde(default)]
    pub name: String,

    /// Insight value
    #[serde(default)]
    pub value: String,
}
