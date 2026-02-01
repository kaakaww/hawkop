//! Functional tests for hosted scanning features (run, config, env, oas)
//!
//! These tests verify the hosted scanning commands work correctly against the real API.
//! Many of these commands require feature flags that may not be enabled in all organizations.

use predicates::prelude::*;

use super::FunctionalTestContext;

// ============================================================================
// Run Commands (Scanner Execution)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_status_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // run status requires --app
    ctx.run(&["run", "status"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_status_with_nonexistent_app() {
    let ctx = FunctionalTestContext::new();

    // Should fail with app not found
    ctx.run(&["run", "status", "--app", "nonexistent-app-12345"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Application not found")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_start_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // run start requires --app
    ctx.run(&["run", "start"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_stop_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // run stop requires --app
    ctx.run(&["run", "stop"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["run", "--help"])
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("status"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_start_help_shows_options() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["run", "start", "--help"])
        .success()
        .stdout(predicate::str::contains("--app"))
        .stdout(predicate::str::contains("--env"))
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("--watch"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_status_help_shows_options() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["run", "status", "--help"])
        .success()
        .stdout(predicate::str::contains("--app"))
        .stdout(predicate::str::contains("--watch"))
        .stdout(predicate::str::contains("--interval"));
}

// ============================================================================
// Config Commands (Configuration Management)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_get_nonexistent() {
    let ctx = FunctionalTestContext::new();

    // Should fail with not found or access denied if feature not enabled
    ctx.run(&["config", "get", "nonexistent-config-12345"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Access denied")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_delete_nonexistent() {
    let ctx = FunctionalTestContext::new();

    // The StackHawk API implements idempotent DELETE - it returns success
    // even when the resource doesn't exist. This is valid REST behavior.
    // The command should succeed (or fail with access denied if feature not enabled)
    let result = ctx
        .command(&["config", "delete", "nonexistent-config-12345", "--yes"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Either succeeds (idempotent delete) or fails with access denied
    if !result.status.success() {
        assert!(
            stderr.contains("Access denied"),
            "Expected success (idempotent delete) or 'Access denied', got: {}",
            stderr
        );
    }
    // Success is acceptable - idempotent DELETE behavior
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_set_missing_file_flag() {
    let ctx = FunctionalTestContext::new();

    // config set requires --file
    ctx.run(&["config", "set", "test-config"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_set_file_not_found() {
    let ctx = FunctionalTestContext::new();

    // Should fail with file not found
    ctx.run(&[
        "config",
        "set",
        "test-config",
        "--file",
        "/nonexistent/file.yml",
    ])
    .failure()
    .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_validate_missing_args() {
    let ctx = FunctionalTestContext::new();

    // config validate requires either name or --file
    // Actually, it succeeds with a message saying to specify one
    let result = ctx.run(&["config", "validate"]);
    // This should fail because neither name nor --file is provided
    result.failure();
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_validate_file_not_found() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["config", "validate", "--file", "/nonexistent/file.yml"])
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_rename_nonexistent() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["config", "rename", "nonexistent-config-12345", "new-name"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Access denied")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_config_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["config", "--help"])
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("rename"))
        .stdout(predicate::str::contains("validate"));
}

// ============================================================================
// Env Commands (Environment Management)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_list_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // env list requires --app
    ctx.run(&["env", "list"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_list_with_nonexistent_app() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["env", "list", "--app", "nonexistent-app-12345"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Application not found")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_config_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // env config requires --app
    ctx.run(&["env", "config", "production"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_create_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // env create requires --app
    ctx.run(&["env", "create", "test-env"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_delete_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // env delete requires --app
    ctx.run(&["env", "delete", "test-env"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["env", "--help"])
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

// ============================================================================
// OAS Commands (Extended)
// ============================================================================

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_get_without_id_fails() {
    let ctx = FunctionalTestContext::new();

    // oas get requires an ID
    ctx.run(&["oas", "get"])
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("missing")));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_get_nonexistent() {
    let ctx = FunctionalTestContext::new();

    // Should fail with not found or access denied
    ctx.run(&["oas", "get", "00000000-0000-0000-0000-000000000000"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Access denied")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_mappings_without_app_fails() {
    let ctx = FunctionalTestContext::new();

    // oas mappings requires --app
    ctx.run(&["oas", "mappings"])
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_mappings_with_nonexistent_app() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["oas", "mappings", "--app", "nonexistent-app-12345"])
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Not found"))
                .or(predicate::str::contains("Application not found")),
        );
}

#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_help_shows_subcommands() {
    let ctx = FunctionalTestContext::new();

    ctx.run(&["oas", "--help"])
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("mappings"));
}

// ============================================================================
// Feature Flag Dependent Tests (Real API calls)
// ============================================================================

/// Test that run status works with a real app (if hosted scanning is available)
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_run_commands_feature_flag_check() {
    let ctx = FunctionalTestContext::new();

    // First get an app to test with
    let output = ctx
        .command(&["app", "list", "--format", "json", "--limit", "1"])
        .output()
        .expect("Failed to list apps");

    if !output.status.success() {
        eprintln!("[SKIP] Could not list apps to find a test target");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to parse JSON to get an app name
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(apps) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(first_app) = apps.first() {
                if let Some(app_name) = first_app.get("name").and_then(|n| n.as_str()) {
                    // Try run status with this app
                    ctx.run_feature_flag_dependent(
                        &["run", "status", "--app", app_name],
                        "hosted-scanning",
                    );
                    return;
                }
            }
        }
    }

    eprintln!("[SKIP] No apps found to test run status");
}

/// Test that env list works with a real app (if environment management is available)
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_env_commands_feature_flag_check() {
    let ctx = FunctionalTestContext::new();

    // First get an app to test with
    let output = ctx
        .command(&["app", "list", "--format", "json", "--limit", "1"])
        .output()
        .expect("Failed to list apps");

    if !output.status.success() {
        eprintln!("[SKIP] Could not list apps to find a test target");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to parse JSON to get an app name
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(apps) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(first_app) = apps.first() {
                if let Some(app_name) = first_app.get("name").and_then(|n| n.as_str()) {
                    // Try env list with this app
                    ctx.run_feature_flag_dependent(
                        &["env", "list", "--app", app_name],
                        "environment-management",
                    );
                    return;
                }
            }
        }
    }

    eprintln!("[SKIP] No apps found to test env commands");
}

/// Test that oas mappings works with a real app
#[test]
#[cfg_attr(not(feature = "functional-tests"), ignore)]
fn test_oas_mappings_feature_flag_check() {
    let ctx = FunctionalTestContext::new();

    // First get an app to test with
    let output = ctx
        .command(&["app", "list", "--format", "json", "--limit", "1"])
        .output()
        .expect("Failed to list apps");

    if !output.status.success() {
        eprintln!("[SKIP] Could not list apps to find a test target");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to parse JSON to get an app name
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(apps) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(first_app) = apps.first() {
                if let Some(app_name) = first_app.get("name").and_then(|n| n.as_str()) {
                    // Try oas mappings with this app
                    ctx.run_feature_flag_dependent(
                        &["oas", "mappings", "--app", app_name],
                        "hosted-oas",
                    );
                    return;
                }
            }
        }
    }

    eprintln!("[SKIP] No apps found to test oas mappings");
}
