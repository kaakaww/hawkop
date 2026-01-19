//! User and team models

use serde::{Deserialize, Serialize};

/// Organization member/user (wrapper for API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// User details from external field
    pub external: UserExternal,
}

// ============================================================================
// Team Detail Models (for CRUD operations)
// ============================================================================

/// Full team detail including members and applications.
///
/// This is returned by the get team endpoint and contains the complete
/// team information with nested users and applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDetail {
    /// Team ID (UUID)
    pub id: String,

    /// Team name
    pub name: String,

    /// Organization ID that owns this team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// Users who are members of this team
    #[serde(default)]
    pub users: Vec<TeamUser>,

    /// Applications assigned to this team
    #[serde(default)]
    pub applications: Vec<TeamApplication>,
}

/// A user who is a member of a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamUser {
    /// User ID (UUID)
    pub user_id: String,

    /// User's display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// User's email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// User's role in the team (e.g., "ADMIN", "MEMBER")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// An application assigned to a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamApplication {
    /// Application ID (UUID)
    pub application_id: String,

    /// Application name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,

    /// Application environments
    #[serde(default)]
    pub environments: Vec<String>,
}

// ============================================================================
// Team Request Models (for create/update operations)
// ============================================================================

/// Request to create a new team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTeamRequest {
    /// Team name
    pub name: String,

    /// Organization ID
    pub organization_id: String,

    /// Initial member user IDs (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_ids: Option<Vec<String>>,

    /// Initial application IDs (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_ids: Option<Vec<String>>,
}

/// Request to update an existing team.
///
/// Note: `team_id` is passed in the URL path, not the body.
/// The API marks teamId and organizationId as readOnly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeamRequest {
    /// Team ID (for internal use - not serialized to request body)
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub team_id: String,

    /// Updated team name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Updated list of member user IDs (replaces existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_ids: Option<Vec<String>>,

    /// Updated list of application IDs (replaces existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_ids: Option<Vec<String>>,
}

/// Request to assign an application to a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UpdateApplicationTeamRequest {
    /// Application ID to assign
    pub application_id: String,

    /// Team ID to assign the application to
    pub team_id: String,
}

/// User details from the external field in API response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExternal {
    /// User ID
    pub id: String,

    /// User email address
    pub email: String,

    /// User's first name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// User's last name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// User's full name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
}

/// Organization team
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Team ID
    pub id: String,

    /// Team name
    pub name: String,

    /// Organization ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}
