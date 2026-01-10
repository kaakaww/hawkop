//! Audit log models

use serde::{Deserialize, Serialize};

/// Audit log record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    /// Unique audit record ID
    #[serde(default)]
    pub id: String,

    /// User activity type (e.g., SCAN_STARTED, APPLICATION_ADDED)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_activity_type: Option<String>,

    /// Organization activity type (e.g., ORGANIZATION_CREATED)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_activity_type: Option<String>,

    /// Organization ID
    #[serde(default)]
    pub organization_id: String,

    /// User ID who performed the action
    #[serde(default)]
    pub user_id: String,

    /// User name
    #[serde(default)]
    pub user_name: String,

    /// User email
    #[serde(default)]
    pub user_email: String,

    /// Payload containing action-specific details (JSON string)
    #[serde(default)]
    pub payload: String,

    /// Timestamp in milliseconds (as string from API)
    #[serde(default)]
    pub timestamp: String,

    /// User IP address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_ip_addr: Option<String>,
}

/// Filter parameters for audit log queries
#[derive(Debug, Clone, Default)]
pub struct AuditFilterParams {
    /// Filter by user activity types
    pub types: Vec<String>,
    /// Filter by organization activity types
    pub org_types: Vec<String>,
    /// Filter by user name
    pub name: Option<String>,
    /// Filter by user email
    pub email: Option<String>,
    /// Start timestamp (milliseconds)
    pub start: Option<i64>,
    /// End timestamp (milliseconds)
    pub end: Option<i64>,
    /// Sort direction (asc/desc)
    pub sort_dir: Option<String>,
    /// Page size (max 1000)
    pub page_size: Option<usize>,
    /// Page token for pagination
    pub page_token: Option<String>,
}

impl AuditFilterParams {
    /// Create new empty filter params
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to query parameters for the API
    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        // Add user activity types (comma-separated)
        if !self.types.is_empty() {
            params.push(("types", self.types.join(",")));
        }

        // Add org activity types (comma-separated)
        if !self.org_types.is_empty() {
            params.push(("orgTypes", self.org_types.join(",")));
        }

        if let Some(ref name) = self.name {
            params.push(("name", name.clone()));
        }

        if let Some(ref email) = self.email {
            params.push(("email", email.clone()));
        }

        if let Some(start) = self.start {
            params.push(("start", start.to_string()));
        }

        if let Some(end) = self.end {
            params.push(("end", end.to_string()));
        }

        // Only supported sort field is "createdDate"
        params.push(("sortField", "createdDate".to_string()));

        if let Some(ref dir) = self.sort_dir {
            params.push(("sortDir", dir.clone()));
        }

        if let Some(size) = self.page_size {
            params.push(("pageSize", size.to_string()));
        }

        if let Some(ref token) = self.page_token {
            params.push(("pageToken", token.clone()));
        }

        params
    }
}
