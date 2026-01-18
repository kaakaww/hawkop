//! Team API trait for CRUD operations
//!
//! This trait covers all team management operations including:
//! - Get team details
//! - Create new teams
//! - Update team properties and membership
//! - Delete teams

use async_trait::async_trait;

use crate::client::models::{
    CreateTeamRequest, TeamDetail, UpdateApplicationTeamRequest, UpdateTeamRequest,
};
use crate::error::Result;

/// Team management operations for the StackHawk API
///
/// This trait covers CRUD operations for teams, including member and
/// application assignment management.
#[async_trait]
#[allow(dead_code)]
pub trait TeamApi: Send + Sync {
    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Get detailed team information including members and applications.
    ///
    /// Returns the full team record with nested users and applications.
    /// May return cached data if caching is enabled.
    async fn get_team(&self, org_id: &str, team_id: &str) -> Result<TeamDetail>;

    /// Get team details, bypassing the cache.
    ///
    /// Use this before mutations to ensure you're working with the latest data.
    /// The default implementation just calls `get_team()` since only the
    /// cached wrapper needs special handling.
    async fn get_team_fresh(&self, org_id: &str, team_id: &str) -> Result<TeamDetail> {
        self.get_team(org_id, team_id).await
    }

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Create a new team in the organization.
    ///
    /// The team can be created with initial members and application assignments.
    async fn create_team(&self, org_id: &str, request: CreateTeamRequest) -> Result<TeamDetail>;

    /// Update an existing team.
    ///
    /// Can update the team name, member list, and application assignments.
    /// User and application ID lists are complete replacements, not incremental.
    async fn update_team(
        &self,
        org_id: &str,
        team_id: &str,
        request: UpdateTeamRequest,
    ) -> Result<TeamDetail>;

    /// Delete a team from the organization.
    ///
    /// This removes the team and unassigns all applications from it.
    /// Members are not deleted from the organization, just removed from the team.
    async fn delete_team(&self, org_id: &str, team_id: &str) -> Result<()>;

    /// Assign an application to a team.
    ///
    /// Changes the team ownership for an application. Only users who belong
    /// to the target team can call this endpoint.
    async fn assign_app_to_team(
        &self,
        org_id: &str,
        team_id: &str,
        request: UpdateApplicationTeamRequest,
    ) -> Result<()>;
}
