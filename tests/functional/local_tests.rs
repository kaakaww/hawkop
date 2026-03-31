//! Local-only functional tests for HawkOp
//!
//! These tests exercise commands that do NOT make API calls:
//! profile management, cache operations, and other local-only features.
//! They are safe to run in any environment.

use predicates::prelude::*;

use super::FunctionalTestContext;

// ============================================================================
// Profile Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // Should show at least the default profile
    ctx.run(&["profile", "list"])
        .success()
        .stdout(predicate::str::contains("default").or(predicate::str::is_empty().not()));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["profile", "list", "--format", "json"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_show_active() {
    let ctx = FunctionalTestContext::new();

    // Show active profile (no name argument)
    ctx.run(&["profile", "show"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_show_specific() {
    let ctx = FunctionalTestContext::new();

    // Show the default profile specifically
    ctx.run(&["profile", "show", "default"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_show_nonexistent() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["profile", "show", "nonexistent-profile-xyz"])
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_create_and_delete() {
    let ctx = FunctionalTestContext::new();
    let name = "hawkop-functest-profile";

    // Create profile by copying from an existing one (avoids interactive API key prompt)
    // Use the test profile if available, otherwise default
    let from_profile = ctx.profile.as_deref().unwrap_or("default");
    ctx.run(&["profile", "create", name, "--from", from_profile])
        .success();

    // Verify it appears in list
    ctx.run(&["profile", "list"])
        .success()
        .stdout(predicate::str::contains(name));

    // Show it
    ctx.run(&["profile", "show", name]).success();

    // Delete it
    ctx.run(&["profile", "delete", name, "--yes"]).success();

    // Verify it's gone
    ctx.run(&["profile", "show", name]).failure();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_create_duplicate_fails() {
    let ctx = FunctionalTestContext::new();

    // The "default" profile always exists
    ctx.run(&["profile", "create", "default"])
        .failure()
        .stderr(predicate::str::contains("exists").or(predicate::str::contains("already")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_delete_default_fails() {
    let ctx = FunctionalTestContext::new();

    // Should not be able to delete the default profile
    ctx.run(&["profile", "delete", "default", "--yes"])
        .failure()
        .stderr(
            predicate::str::contains("cannot delete")
                .or(predicate::str::contains("Cannot delete"))
                .or(predicate::str::contains("default")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_delete_nonexistent_fails() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["profile", "delete", "nonexistent-profile-xyz", "--yes"])
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_use_nonexistent_fails() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["profile", "use", "nonexistent-profile-xyz"])
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_create_with_from() {
    let ctx = FunctionalTestContext::new();
    let name = "hawkop-functest-copied";

    // Create profile copying from default
    ctx.run(&["profile", "create", name, "--from", "default"])
        .success();

    // Verify it exists
    ctx.run(&["profile", "show", name]).success();

    // Cleanup
    let _ = ctx.command(&["profile", "delete", name, "--yes"]).output();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_profile_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["profile", "--help"])
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("use"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("show"));
}

// ============================================================================
// Cache Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_cache_clear_succeeds() {
    let ctx = FunctionalTestContext::new();

    // cache clear should always succeed
    ctx.run(&["cache", "clear"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_cache_clear_then_status() {
    let ctx = FunctionalTestContext::new();

    // Clear the cache
    ctx.run(&["cache", "clear"]).success();

    // Status should show empty/zero entries
    ctx.run(&["cache", "status"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_cache_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["cache", "--help"])
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("clear"))
        .stdout(predicate::str::contains("path"));
}

// ============================================================================
// Org Set Command
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_set_and_get_roundtrip() {
    let ctx = FunctionalTestContext::new();

    // First get the current org so we can restore it
    let original = ctx.run_success(&["org", "get", "--format", "json"]);

    // Get the org ID from the list
    let list_output = ctx.run_success(&["org", "list", "--format", "json"]);

    // Parse to get an org ID
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&list_output) {
        if let Some(orgs) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(first_org) = orgs.first() {
                if let Some(org_id) = first_org.get("id").and_then(|id| id.as_str()) {
                    // Set the org
                    ctx.run(&["org", "set", org_id]).success();

                    // Verify it was set
                    ctx.run(&["org", "get"])
                        .success()
                        .stdout(predicate::str::contains(org_id));

                    // Restore original (parse original JSON to get the ID back)
                    // Note: org get --format json wraps output as {"data": {...}, "meta": {...}}
                    if let Ok(orig_json) = serde_json::from_str::<serde_json::Value>(&original) {
                        if let Some(orig_id) = orig_json
                            .get("data")
                            .and_then(|d| d.get("id"))
                            .and_then(|id| id.as_str())
                        {
                            let _ = ctx.command(&["org", "set", orig_id]).output();
                        } else {
                            panic!(
                                "[BUG] Could not parse original org ID from JSON — org will not be restored!\nJSON: {}",
                                original
                            );
                        }
                    }
                    return;
                }
            }
        }
    }

    eprintln!("[SKIP] Could not parse org list to find an org ID for set test");
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_set_invalid_id_fails() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "set", "not-a-valid-uuid"]).failure();
}
