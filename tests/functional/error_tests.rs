//! Error scenario functional tests for HawkOp
//!
//! These tests verify that HawkOp returns appropriate, actionable error messages
//! when operations fail. Good error messages help users understand what went wrong
//! and how to fix it.

use predicates::prelude::*;

use super::FunctionalTestContext;

// ============================================================================
// Invalid Organization ID Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_invalid_org_id_returns_helpful_error() {
    let ctx = FunctionalTestContext::new();

    // Use an obviously invalid org ID
    ctx.run(&["app", "list", "--org", "invalid-org-id-12345"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_malformed_uuid_org_id() {
    let ctx = FunctionalTestContext::new();

    // Use a malformed UUID (missing segments)
    ctx.run(&["app", "list", "--org", "not-a-uuid"]).failure();
}

// ============================================================================
// Invalid Team Identifier Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_nonexistent_team_returns_not_found() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["team", "get", "hawkop-functest-nonexistent-team-99999"])
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_nonexistent_team_delete_returns_not_found() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&[
        "team",
        "delete",
        "hawkop-functest-nonexistent-team-99999",
        "--yes",
    ])
    .failure()
    .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));
}

// ============================================================================
// Invalid Scan ID Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_nonexistent_scan_id_returns_error() {
    let ctx = FunctionalTestContext::new();

    // Use a clearly fake scan ID
    ctx.run(&["scan", "get", "00000000-0000-0000-0000-000000000000"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("error")),
        );
}

// ============================================================================
// Missing Required Arguments
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_create_missing_name_shows_help() {
    let ctx = FunctionalTestContext::new();

    // Missing required argument should show usage
    ctx.run(&["team", "create"]).failure().stderr(
        predicate::str::contains("Usage")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("argument")),
    );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_get_missing_identifier_shows_help() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["team", "get"]).failure().stderr(
        predicate::str::contains("Usage")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("argument")),
    );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_rename_missing_args_shows_help() {
    let ctx = FunctionalTestContext::new();

    // Missing both old name and new name
    ctx.run(&["team", "rename"]).failure().stderr(
        predicate::str::contains("Usage")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("argument")),
    );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_get_missing_id_shows_help() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["scan", "get"]).failure().stderr(
        predicate::str::contains("Usage")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("argument")),
    );
}

// ============================================================================
// Invalid Command/Subcommand Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_unknown_command_shows_suggestions() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["unknowncommand"]).failure().stderr(
        predicate::str::contains("Invalid")
            .or(predicate::str::contains("error"))
            .or(predicate::str::contains("unrecognized")),
    );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_unknown_subcommand_shows_help() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "unknownsubcommand"]).failure().stderr(
        predicate::str::contains("Invalid")
            .or(predicate::str::contains("error"))
            .or(predicate::str::contains("unrecognized")),
    );
}

// ============================================================================
// Invalid Flag Values
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_invalid_format_value() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "list", "--format", "xml"])
        .failure()
        .stderr(
            predicate::str::contains("invalid").or(predicate::str::contains("possible values")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_invalid_limit_value_non_numeric() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["app", "list", "--limit", "abc"])
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("number")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_negative_limit_value() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["app", "list", "--limit", "-5"])
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("unexpected")));
}

// ============================================================================
// User Resolution Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_nonexistent_user_in_team_add() {
    let ctx = FunctionalTestContext::new();

    // First we need a valid team, so create one
    let name = super::test_resource_name();
    ctx.run(&["team", "create", &name]).success();

    // Try to add a user that doesn't exist
    ctx.run(&[
        "team",
        "add-user",
        &name,
        "nonexistent-user@fake-domain-12345.invalid",
    ])
    .failure()
    .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));

    // Cleanup
    let _ = ctx.command(&["team", "delete", &name, "--yes"]).output();
}

// ============================================================================
// App Resolution Errors
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_nonexistent_app_in_team_add() {
    let ctx = FunctionalTestContext::new();

    // First we need a valid team
    let name = super::test_resource_name();
    ctx.run(&["team", "create", &name]).success();

    // Try to add an app that doesn't exist
    ctx.run(&[
        "team",
        "add-app",
        &name,
        "nonexistent-app-00000000-0000-0000-0000-000000000000",
    ])
    .failure()
    .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));

    // Cleanup
    let _ = ctx.command(&["team", "delete", &name, "--yes"]).output();
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

/// Verify error messages include the problematic identifier for debugging.
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_error_includes_identifier() {
    let ctx = FunctionalTestContext::new();
    let fake_team = "hawkop-functest-definitely-not-a-real-team";

    ctx.run(&["team", "get", fake_team])
        .failure()
        .stderr(predicate::str::contains(fake_team));
}

/// Verify error messages suggest a next step (like listing available resources).
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_not_found_suggests_list() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["team", "get", "hawkop-functest-nonexistent-xyz"])
        .failure()
        .stderr(
            predicate::str::contains("team list").or(predicate::str::contains("hawkop team list")),
        );
}
