use assert_cmd::prelude::*;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn future_timestamp() -> String {
    (Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
}

fn write_config(temp: &PathBuf, org_id: &str) -> PathBuf {
    let path = temp.join("config.yaml");
    let contents = format!(
        "api_key: test-key\norg_id: {org_id}\njwt:\n  token: dummy\n  expires_at: {}\npreferences:\n  page_size: 1000\n",
        future_timestamp()
    );
    fs::write(&path, contents).expect("failed to write config");
    path
}

#[test]
fn status_uses_custom_config_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-status");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("status")
        .arg("--config")
        .arg(&config_path)
        .env_remove("HAWKOP_CONFIG")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Default organization: org-status"));
    assert!(stdout.contains(&config_path.to_string_lossy().to_string()));

    Ok(())
}

#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn org_get_prefers_runtime_org_override() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    let _orgs = server
        .mock("GET", "/api/v1/user")
        .with_status(200)
        .with_body(
            r#"{
                "user": {
                    "external": {
                        "organizations": [
                            { "organization": { "id": "override-org", "name": "Override Org" } }
                        ]
                    }
                }
            }"#,
        )
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "config-org");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("org")
        .arg("get")
        .arg("--org")
        .arg("override-org")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("override-org"));
    assert!(stdout.contains("Override Org"));

    Ok(())
}

#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn org_set_updates_custom_config_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    let _orgs = server
        .mock("GET", "/api/v1/user")
        .with_status(200)
        .with_body(
            r#"{
                "user": {
                    "external": {
                        "organizations": [
                            { "organization": { "id": "new-org", "name": "New Org" } }
                        ]
                    }
                }
            }"#,
        )
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "old-org");

    Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("set")
        .arg("new-org")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .success();

    let saved = fs::read_to_string(config_path)?;
    assert!(saved.contains("new-org"));
    Ok(())
}

#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn app_list_uses_v2_base_url() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    let _orgs = server
        .mock("GET", "/api/v1/user")
        .with_status(200)
        .with_body(
            r#"{
                "user": {
                    "external": {
                        "organizations": [
                            { "organization": { "id": "org-123", "name": "Org 123" } }
                        ]
                    }
                }
            }"#,
        )
        .create();

    let _apps = server
        .mock("GET", "/api/v2/org/org-123/apps")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(
            r#"{
                "applications": [
                    { "applicationId": "app-1", "name": "App One" }
                ]
            }"#,
        )
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-123");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("app")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("App One"));
    assert!(stdout.contains("app-1"));
    assert!(stdout.contains("\"meta\""));

    Ok(())
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

/// Test that missing config file shows actionable error message.
#[test]
fn missing_config_shows_helpful_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let nonexistent_config = temp.path().join("does-not-exist.yaml");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("list")
        .arg("--config")
        .arg(&nonexistent_config)
        .env_remove("HAWKOP_CONFIG")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should suggest running init
    assert!(
        stderr.contains("hawkop init"),
        "Expected error to mention 'hawkop init', got: {}",
        stderr
    );

    Ok(())
}

/// Test that invalid org ID returns a clear not-found error.
#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn invalid_org_id_returns_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    // Mock user endpoint to return orgs that don't include the requested one
    let _orgs = server
        .mock("GET", "/api/v1/user")
        .with_status(200)
        .with_body(
            r#"{
                "user": {
                    "external": {
                        "organizations": [
                            { "organization": { "id": "real-org", "name": "Real Org" } }
                        ]
                    }
                }
            }"#,
        )
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "nonexistent-org-xyz");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("get")
        .arg("--org")
        .arg("nonexistent-org-xyz")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should mention the org wasn't found and include the ID
    assert!(
        stderr.contains("not found") || stderr.contains("Not found"),
        "Expected error to mention 'not found', got: {}",
        stderr
    );
    assert!(
        stderr.contains("nonexistent-org-xyz"),
        "Expected error to include org ID 'nonexistent-org-xyz', got: {}",
        stderr
    );

    Ok(())
}

/// Test that 401 Unauthorized shows actionable error.
#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn unauthorized_error_suggests_init() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    // Mock user endpoint to return 401 (unauthorized)
    // This simulates an invalid/expired JWT token
    let _user = server
        .mock("GET", "/api/v1/user")
        .with_status(401)
        .with_body(r#"{"error": "Unauthorized"}"#)
        .create();

    // Mock the auth endpoint (GET not POST!) to return 401 for re-auth attempts
    let _auth = server
        .mock("GET", "/api/v1/auth/login")
        .with_status(401)
        .with_body(r#"{"error": "Invalid API key"}"#)
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-123");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should suggest running init for auth issues
    assert!(
        stderr.contains("hawkop init") || stderr.contains("Authentication"),
        "Expected error to mention 'hawkop init' or 'Authentication', got: {}",
        stderr
    );

    Ok(())
}

/// Test that rate limit (429) response shows retry message after exhausting retries.
#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn rate_limit_shows_retry_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    // Mock user endpoint to always return 429
    // The client will retry with exponential backoff, but eventually give up
    let _rate_limited = server
        .mock("GET", "/api/v1/user")
        .with_status(429)
        .with_header("retry-after", "0") // 0 seconds for fast test
        .expect_at_least(1)
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-123");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should indicate rate limiting
    assert!(
        stderr.to_lowercase().contains("rate limit") || stderr.to_lowercase().contains("try again"),
        "Expected error to mention rate limit, got: {}",
        stderr
    );

    Ok(())
}

/// Test that 500 server error shows server error message.
#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn server_error_shows_helpful_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let api_host = server.url();

    // Mock user endpoint to return 500
    let _server_error = server
        .mock("GET", "/api/v1/user")
        .with_status(500)
        .with_body(r#"{"error": "Internal server error"}"#)
        .create();

    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-123");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", &api_host)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should indicate a server error
    assert!(
        stderr.to_lowercase().contains("server")
            || stderr.to_lowercase().contains("error")
            || stderr.contains("500"),
        "Expected error to mention server error, got: {}",
        stderr
    );

    Ok(())
}

/// Test that network connection errors show helpful message.
#[test]
fn connection_error_shows_network_message() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let config_path = write_config(&temp.path().to_path_buf(), "org-123");

    // Point to a port that nothing is listening on
    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")
        .arg("org")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_HOST", "http://127.0.0.1:59999")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    // Should indicate a network/connection error
    assert!(
        stderr.to_lowercase().contains("network")
            || stderr.to_lowercase().contains("connect")
            || stderr.to_lowercase().contains("error"),
        "Expected error to mention network/connection issue, got: {}",
        stderr
    );

    Ok(())
}
