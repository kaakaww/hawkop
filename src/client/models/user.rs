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
/// ⚠️  CRITICAL: This is a REPLACE-ALL API, not a PATCH API!
///
/// Despite OpenAPI marking teamId/organizationId as "readOnly", the StackHawk
/// API requires ALL 5 fields to be present. Any missing field defaults to an
/// empty string or empty array, which will ERASE that data.
///
/// Before any PUT to /api/v1/org/{orgId}/team/{teamId}:
/// 1. Fetch the current team state (GET the team first)
/// 2. Build the complete desired state locally
/// 3. PUT the entire state back
///
/// See: .claude/skills/stackhawk-api-sherpa/api-quirks.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeamRequest {
    /// Team ID - REQUIRED despite OpenAPI saying "readOnly"
    pub team_id: String,

    /// Organization ID - REQUIRED despite OpenAPI saying "readOnly"
    pub organization_id: String,

    /// Team name - REQUIRED, defaults to "" if not provided
    pub name: Option<String>,

    /// Complete list of user IDs - REQUIRED, defaults to [] if not provided
    /// This REPLACES all team members, not appends!
    pub user_ids: Option<Vec<String>>,

    /// Complete list of application IDs - REQUIRED, defaults to [] if not provided
    /// This REPLACES all app assignments, not appends!
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_team_request_includes_all_fields() {
        // CRITICAL: All 5 fields must be sent to avoid data loss
        let req = UpdateTeamRequest {
            team_id: "team-uuid-123".to_string(),
            organization_id: "org-uuid-456".to_string(),
            name: Some("New Name".to_string()),
            user_ids: Some(vec!["user-1".to_string()]),
            application_ids: Some(vec!["app-1".to_string()]),
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        println!("Serialized JSON (all 5 fields required):\n{}", json);

        // ALL fields must be in the serialized output
        assert!(json.contains("teamId"), "teamId MUST be serialized");
        assert!(
            json.contains("team-uuid-123"),
            "teamId value should be in JSON"
        );
        assert!(
            json.contains("organizationId"),
            "organizationId MUST be serialized"
        );
        assert!(
            json.contains("org-uuid-456"),
            "organizationId value should be in JSON"
        );
        assert!(json.contains("name"), "name MUST be serialized");
        assert!(json.contains("New Name"), "name value should be in JSON");
        assert!(json.contains("userIds"), "userIds MUST be serialized");
        assert!(
            json.contains("applicationIds"),
            "applicationIds MUST be serialized"
        );
    }

    #[test]
    fn test_update_team_request_with_empty_arrays() {
        // Even empty arrays must be explicitly sent
        let req = UpdateTeamRequest {
            team_id: "team-123".to_string(),
            organization_id: "org-456".to_string(),
            name: Some("Team Name".to_string()),
            user_ids: Some(vec![]),
            application_ids: Some(vec![]),
        };

        let json = serde_json::to_string_pretty(&req).unwrap();
        println!("Serialized JSON with empty arrays:\n{}", json);

        // All fields present
        assert!(json.contains("teamId"), "teamId MUST be serialized");
        assert!(
            json.contains("organizationId"),
            "organizationId MUST be serialized"
        );
        assert!(json.contains("name"), "name MUST be serialized");
        assert!(json.contains("userIds"), "userIds MUST be serialized");
        assert!(
            json.contains("applicationIds"),
            "applicationIds MUST be serialized"
        );
    }
}
