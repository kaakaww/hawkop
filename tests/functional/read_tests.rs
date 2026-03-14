//! Read-only functional tests for HawkOp
//!
//! These tests verify that read operations work correctly against the real API.
//! They do not modify any data and are safe to run against any environment.

use predicates::prelude::*;

use super::FunctionalTestContext;

// ============================================================================
// Status Command
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_status_shows_config() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["status"])
        .success()
        .stdout(predicate::str::contains("Configuration"));
}

// NOTE: test_status_json_format was removed because the status command
// does not support --format json - it always outputs human-readable text

// ============================================================================
// Organization Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_list_returns_orgs() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_get_shows_current_org() {
    let ctx = FunctionalTestContext::new();

    // Should show the current organization
    ctx.run(&["org", "get"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_org_get_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["org", "get", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("\"name\""));
}

// ============================================================================
// Application Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_app_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["app", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_app_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["app", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_app_list_with_limit() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["app", "list", "--limit", "5"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_app_list_with_type_filter() {
    let ctx = FunctionalTestContext::new();

    // Filter by application type
    ctx.run(&["app", "list", "--type", "cloud"]).success();
}

// ============================================================================
// Scan Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["scan", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["scan", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_list_with_limit() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["scan", "list", "--limit", "5"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_list_with_status_filter() {
    let ctx = FunctionalTestContext::new();

    // Filter by scan status
    ctx.run(&["scan", "list", "--status", "COMPLETED"])
        .success();
}

// ============================================================================
// Scan Get (Happy Path - Latest Scan)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_get_latest_succeeds() {
    let ctx = FunctionalTestContext::new();

    // `scan get` without an ID defaults to "latest"
    // This may fail if there are no scans, but that's an expected condition
    let output = ctx
        .command(&["scan", "get"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain scan information
        assert!(
            stdout.contains("Scan") || stdout.contains("scan") || stdout.contains("Alert"),
            "Expected scan details in output"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable: no scans found
        assert!(
            stderr.contains("No scans found") || stderr.contains("not found"),
            "Expected 'no scans' or 'not found', got: {}",
            stderr
        );
    }
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_get_latest_json_format() {
    let ctx = FunctionalTestContext::new();

    let output = ctx
        .command(&["scan", "get", "--format", "json"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // JSON output should be parseable
        assert!(
            serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
            "Expected valid JSON output"
        );
    }
    // If no scans exist, failure is acceptable
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_scan_get_with_app_filter() {
    let ctx = FunctionalTestContext::new();

    // Get an app name to filter by
    let output = ctx
        .command(&["app", "list", "--format", "json", "--limit", "1"])
        .output()
        .expect("Failed to list apps");

    if !output.status.success() {
        eprintln!("[SKIP] Could not list apps");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(apps) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(first_app) = apps.first() {
                if let Some(app_name) = first_app.get("name").and_then(|n| n.as_str()) {
                    // scan get --app <name> should work (may have no scans)
                    let result = ctx
                        .command(&["scan", "get", "--app", app_name])
                        .output()
                        .expect("Failed to run scan get");

                    // Either success or "no scans found" is acceptable
                    if !result.status.success() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        assert!(
                            stderr.contains("No scans found") || stderr.contains("not found"),
                            "Expected 'no scans' for app filter, got: {}",
                            stderr
                        );
                    }
                    return;
                }
            }
        }
    }
    eprintln!("[SKIP] No apps found for scan get --app test");
}

// ============================================================================
// Audit List with Filters
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_audit_list_with_type_filter() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["audit", "list", "--type", "SCAN_STARTED", "--limit", "5"])
        .success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_audit_list_with_date_range() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["audit", "list", "--since", "30d", "--limit", "5"])
        .success();
}

// ============================================================================
// User Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_user_list_returns_users() {
    let ctx = FunctionalTestContext::new();

    // Should return at least one user (the authenticated user)
    ctx.run(&["user", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_user_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["user", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

// ============================================================================
// Team Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["team", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["team", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_team_list_with_filters() {
    let ctx = FunctionalTestContext::new();

    // Filter by name substring
    ctx.run(&["team", "list", "--name", "test"]).success();
}

// ============================================================================
// Policy Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_policy_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["policy", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_policy_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["policy", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

// ============================================================================
// Repository Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_repo_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["repo", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_repo_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["repo", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

// ============================================================================
// OAS (OpenAPI Spec) Commands (may require feature flag)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, or fail with Access denied if feature not enabled
    ctx.run_feature_flag_dependent(&["oas", "list"], "hosted-oas");
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_list_json_format() {
    let ctx = FunctionalTestContext::new();

    // May fail with Access denied if feature not enabled
    ctx.run_feature_flag_dependent(&["oas", "list", "--format", "json"], "hosted-oas");
}

// ============================================================================
// Config Commands (requires hosted scan configs feature)
// ============================================================================
// NOTE: The config list endpoint requires hosted scan configs feature which
// may not be available in all organizations. Tests will pass with a warning
// if the feature is not enabled.

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, or fail with Access denied if feature not enabled
    ctx.run_feature_flag_dependent(&["config", "list"], "hosted-scan-configs");
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_list_json_format() {
    let ctx = FunctionalTestContext::new();

    // May fail with Access denied if feature not enabled
    ctx.run_feature_flag_dependent(
        &["config", "list", "--format", "json"],
        "hosted-scan-configs",
    );
}

// ============================================================================
// Secret Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_secret_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // May return empty list, but should succeed
    ctx.run(&["secret", "list"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_secret_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["secret", "list", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

// ============================================================================
// Audit Commands
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_audit_list_succeeds() {
    let ctx = FunctionalTestContext::new();

    // Should return audit entries (at least from test setup)
    ctx.run(&["audit", "list", "--limit", "10"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_audit_list_json_format() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["audit", "list", "--limit", "10", "--format", "json"])
        .success()
        .stdout(predicate::str::contains("\"data\""))
        .stdout(predicate::str::contains("\"meta\""));
}

// ============================================================================
// Cache Commands (Local-only - No API calls)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_cache_status_succeeds() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["cache", "status"]).success();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_cache_path_shows_path() {
    let ctx = FunctionalTestContext::new();

    // cache path shows the cache directory, not the database file
    ctx.run(&["cache", "path"])
        .success()
        .stdout(predicate::str::contains("hawkop"));
}

// ============================================================================
// Version Command (Local-only)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_version_shows_version() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["version"])
        .success()
        .stdout(predicate::str::contains("hawkop"));
}

// ============================================================================
// Help Command (Local-only)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_help_shows_commands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["--help"])
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("org"))
        .stdout(predicate::str::contains("app"))
        .stdout(predicate::str::contains("scan"));
}

// ============================================================================
// Completion Command (Local-only)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_completion_bash() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["completion", "bash"])
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_completion_zsh() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["completion", "zsh"])
        .success()
        .stdout(predicate::str::contains("compdef"));
}
