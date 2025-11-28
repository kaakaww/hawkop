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
    let base_v1 = format!("{}/api/v1", server.url());

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
        .env("HAWKOP_API_BASE_URL", base_v1)
        .env_remove("HAWKOP_API_BASE_URL_V2")
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
    let base_v1 = format!("{}/api/v1", server.url());

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
        .arg("org")
        .arg("set")
        .arg("new-org")
        .arg("--config")
        .arg(&config_path)
        .env("HAWKOP_API_BASE_URL", base_v1)
        .env_remove("HAWKOP_API_BASE_URL_V2")
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
    let base_v1 = format!("{}/api/v1", server.url());
    let base_v2 = format!("{}/api/v2", server.url());

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
        .arg("app")
        .arg("list")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .env("HAWKOP_API_BASE_URL", base_v1)
        .env("HAWKOP_API_BASE_URL_V2", base_v2)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("App One"));
    assert!(stdout.contains("app-1"));
    assert!(stdout.contains("\"meta\""));

    Ok(())
}
