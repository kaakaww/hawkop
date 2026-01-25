//! Mutation functional tests for HawkOp
//!
//! These tests verify that mutation operations (create, update, delete) work
//! correctly against the real API. Each test creates resources with the
//! `hawkop-functest-*` prefix and cleans them up automatically.
//!
//! **IMPORTANT**: These tests modify data. Use only against test environments
//! unless you explicitly confirm production usage.

use predicates::prelude::*;

use super::{FunctionalTestContext, TEST_RESOURCE_PREFIX, TestTeam, test_resource_name};

// ============================================================================
// Team Create Tests
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_and_auto_cleanup() {
    // TestTeam RAII wrapper handles creation and cleanup
    let team = TestTeam::create();

    if team.created {
        // Verify the team was created
        let ctx = FunctionalTestContext::new();
        ctx.run(&["team", "get", &team.name])
            .success()
            .stdout(predicate::str::contains(&team.name));
    }
    // Team will be automatically deleted when `team` goes out of scope
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_json_output() {
    let ctx = FunctionalTestContext::new();
    let name = test_resource_name();

    // Create team and capture output
    let result = ctx.run(&["team", "create", &name, "--format", "json"]);

    result
        .success()
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains(&name));

    // Cleanup
    let _ = ctx.command(&["team", "delete", &name, "--yes"]).output();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_with_dry_run() {
    let ctx = FunctionalTestContext::new();
    let name = test_resource_name();

    // Dry run should NOT create the team
    ctx.run(&["team", "create", &name, "--dry-run"])
        .success()
        .stderr(predicate::str::contains("DRY RUN"));

    // Verify team was NOT created
    ctx.run(&["team", "get", &name]).failure();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_duplicate_fails() {
    // Create the first team
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Attempt to create a team with the same name should fail
        ctx.run(&["team", "create", &team.name])
            .failure()
            .stderr(predicate::str::contains("already exists"));
    }
}

// ============================================================================
// Team Delete Tests
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_delete_with_yes_flag() {
    let ctx = FunctionalTestContext::new();
    let name = test_resource_name();

    // Create team first
    ctx.run(&["team", "create", &name]).success();

    // Delete with --yes to skip confirmation
    ctx.run(&["team", "delete", &name, "--yes"])
        .success()
        .stderr(predicate::str::contains("deleted"));

    // Verify it's gone
    ctx.run(&["team", "get", &name]).failure();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_delete_with_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Dry run should NOT delete the team
        ctx.run(&["team", "delete", &team.name, "--dry-run"])
            .success()
            .stderr(predicate::str::contains("DRY RUN"));

        // Verify team still exists
        ctx.run(&["team", "get", &team.name]).success();
    }
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_delete_nonexistent_fails() {
    let ctx = FunctionalTestContext::new();

    // Try to delete a team that doesn't exist
    ctx.run(&[
        "team",
        "delete",
        "hawkop-functest-nonexistent-12345",
        "--yes",
    ])
    .failure()
    .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));
}

// ============================================================================
// Team Rename Tests
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_rename_succeeds() {
    let ctx = FunctionalTestContext::new();
    let original_name = test_resource_name();
    let new_name = format!("{}-renamed", original_name);

    // Create team first
    ctx.run(&["team", "create", &original_name]).success();

    // Rename the team
    ctx.run(&["team", "rename", &original_name, &new_name])
        .success()
        .stderr(predicate::str::contains("renamed"));

    // Verify it was renamed (old name not found, new name found)
    ctx.run(&["team", "get", &original_name]).failure();

    ctx.run(&["team", "get", &new_name]).success();

    // Cleanup (delete by new name)
    let _ = ctx
        .command(&["team", "delete", &new_name, "--yes"])
        .output();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_rename_with_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();
        let new_name = format!("{}-would-rename", team.name);

        // Dry run should NOT rename the team
        ctx.run(&["team", "rename", &team.name, &new_name, "--dry-run"])
            .success()
            .stderr(predicate::str::contains("DRY RUN"));

        // Verify original name still works
        ctx.run(&["team", "get", &team.name]).success();
    }
}

// ============================================================================
// Team Get Tests (with created team)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_get_by_name() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        ctx.run(&["team", "get", &team.name])
            .success()
            .stdout(predicate::str::contains(&team.name));
    }
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_get_json_format() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        ctx.run(&["team", "get", &team.name, "--format", "json"])
            .success()
            .stdout(predicate::str::contains("\"name\""))
            .stdout(predicate::str::contains("\"id\""))
            .stdout(predicate::str::contains(&team.name));
    }
}

// ============================================================================
// Team User Management Tests
// ============================================================================

// Note: These tests require a valid user in the org.
// We don't create users via CLI, so these tests verify the command works
// with the existing authenticated user.

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_add_user_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Dry run adding a user (use a fake email to test the flow)
        // This should show the dry run message even if the user doesn't exist
        ctx.run(&[
            "team",
            "add-user",
            &team.name,
            "test@example.com",
            "--dry-run",
        ])
        .success()
        .stderr(predicate::str::contains("DRY RUN"));
    }
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_remove_user_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Dry run removing a user
        ctx.run(&[
            "team",
            "remove-user",
            &team.name,
            "test@example.com",
            "--dry-run",
        ])
        .success()
        .stderr(predicate::str::contains("DRY RUN"));
    }
}

// ============================================================================
// Cleanup Helper Tests
// ============================================================================

/// This test manually verifies cleanup works by listing test teams.
/// If there are orphaned `hawkop-functest-*` teams, they indicate cleanup failures.
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_list_functest_teams_for_manual_review() {
    let ctx = FunctionalTestContext::new();

    // List teams filtered by our test prefix
    // This helps identify any orphaned test resources
    let result = ctx.run(&[
        "team",
        "list",
        "--name",
        TEST_RESOURCE_PREFIX,
        "--format",
        "json",
    ]);

    result.success();

    // Note: This test doesn't assert on the count because there may be
    // legitimately running tests. It's meant for manual inspection.
    eprintln!(
        "[INFO] Check for orphaned test teams with prefix: {}",
        TEST_RESOURCE_PREFIX
    );
}

// ============================================================================
// Team App Management Tests (Dry Run Only)
// ============================================================================

// Note: We don't create apps via CLI, so these tests verify commands work
// in dry-run mode to avoid requiring pre-existing apps.

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_add_app_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Dry run adding an app (use a fake app name)
        ctx.run(&["team", "add-app", &team.name, "fake-app-id", "--dry-run"])
            .success()
            .stderr(predicate::str::contains("DRY RUN"));
    }
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_remove_app_dry_run() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        // Dry run removing an app
        ctx.run(&["team", "remove-app", &team.name, "fake-app-id", "--dry-run"])
            .success()
            .stderr(predicate::str::contains("DRY RUN"));
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_empty_name_fails() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["team", "create", ""])
        .failure()
        .stderr(predicate::str::contains("empty").or(predicate::str::contains("cannot")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_rename_to_empty_fails() {
    let team = TestTeam::create();

    if team.created {
        let ctx = FunctionalTestContext::new();

        ctx.run(&["team", "rename", &team.name, ""])
            .failure()
            .stderr(predicate::str::contains("empty").or(predicate::str::contains("cannot")));
    }
}
