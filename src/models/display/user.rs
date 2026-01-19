//! User and team display models

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::{Team, User, UserExternal};

/// User/member display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct UserDisplay {
    /// User ID
    #[tabled(rename = "USER ID")]
    pub id: String,

    /// User email
    #[tabled(rename = "EMAIL")]
    pub email: String,

    /// User name (first + last)
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Role in organization
    #[tabled(rename = "ROLE")]
    pub role: String,
}

impl From<User> for UserDisplay {
    fn from(user: User) -> Self {
        UserDisplay::from(user.external)
    }
}

impl From<&User> for UserDisplay {
    fn from(user: &User) -> Self {
        UserDisplay::from(user.external.clone())
    }
}

impl From<UserExternal> for UserDisplay {
    fn from(user: UserExternal) -> Self {
        // Prefer full_name if available, otherwise combine first/last
        let name = user
            .full_name
            .unwrap_or_else(|| match (&user.first_name, &user.last_name) {
                (Some(first), Some(last)) => format!("{} {}", first, last),
                (Some(first), None) => first.clone(),
                (None, Some(last)) => last.clone(),
                (None, None) => "--".to_string(),
            });

        Self {
            id: user.id,
            email: user.email,
            name,
            role: "--".to_string(), // Role not available in current API response
        }
    }
}

/// Team display model for table/JSON output (basic, without counts).
/// Note: This is kept for potential future use but TeamListDisplay is preferred.
#[allow(dead_code)]
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct TeamDisplay {
    /// Team ID
    #[tabled(rename = "TEAM ID")]
    pub id: String,

    /// Team name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Team> for TeamDisplay {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
        }
    }
}

impl From<&Team> for TeamDisplay {
    fn from(team: &Team) -> Self {
        TeamDisplay::from(team.clone())
    }
}

use crate::client::models::TeamDetail;

/// Team list display model with user and app counts.
///
/// This is used for `team list` output to show membership and assignment counts.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct TeamListDisplay {
    /// Team ID
    #[tabled(rename = "TEAM ID")]
    pub id: String,

    /// Team name
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Number of users in the team
    #[tabled(rename = "USERS")]
    pub users: usize,

    /// Number of applications assigned to the team
    #[tabled(rename = "APPS")]
    pub apps: usize,
}

impl From<TeamDetail> for TeamListDisplay {
    fn from(team: TeamDetail) -> Self {
        Self {
            id: team.id,
            name: team.name,
            users: team.users.len(),
            apps: team.applications.len(),
        }
    }
}

impl From<&TeamDetail> for TeamListDisplay {
    fn from(team: &TeamDetail) -> Self {
        TeamListDisplay::from(team.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_display_from_team() {
        let team = Team {
            id: "team-123".to_string(),
            name: "Security Team".to_string(),
            organization_id: Some("org-456".to_string()),
        };

        let display = TeamDisplay::from(team);

        assert_eq!(display.id, "team-123");
        assert_eq!(display.name, "Security Team");
    }

    #[test]
    fn test_user_display_from_user() {
        let user = User {
            external: UserExternal {
                id: "user-123".to_string(),
                email: "test@example.com".to_string(),
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                full_name: Some("John Doe".to_string()),
            },
        };

        let display = UserDisplay::from(user);

        assert_eq!(display.id, "user-123");
        assert_eq!(display.email, "test@example.com");
        assert_eq!(display.name, "John Doe");
    }

    #[test]
    fn test_user_display_without_full_name() {
        let user = User {
            external: UserExternal {
                id: "user-456".to_string(),
                email: "jane@example.com".to_string(),
                first_name: Some("Jane".to_string()),
                last_name: Some("Smith".to_string()),
                full_name: None,
            },
        };

        let display = UserDisplay::from(user);

        assert_eq!(display.id, "user-456");
        assert_eq!(display.email, "jane@example.com");
        assert_eq!(display.name, "Jane Smith");
    }
}
