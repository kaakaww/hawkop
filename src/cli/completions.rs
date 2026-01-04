//! Dynamic shell completions for HawkOp CLI
//!
//! Provides TAB completion for scan IDs, app names, plugin IDs, and URI IDs
//! by querying the StackHawk API at completion time.
//!
//! Shell support:
//! - Fish/Zsh: Full support with descriptions
//! - Bash: Values only (no description display)

use std::sync::Arc;
use std::time::Duration;

use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::client::{JwtToken, PaginationParams, StackHawkApi, StackHawkClient};
use crate::config::Config;

/// Maximum number of completion candidates to return
const MAX_COMPLETIONS: usize = 25;

/// Timeout for completion API calls.
/// Using generous timeout for testing - optimize after profiling.
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

/// Create a blocking runtime for completion API calls.
///
/// Completers are called synchronously by the shell, so we need
/// a runtime to execute async API calls.
fn blocking_runtime() -> Option<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()
}

/// Load config and create authenticated client for completions.
///
/// Returns None if config is missing or authentication fails.
/// Completions should never break the shell, so all errors are silent.
fn completion_context() -> Option<(Config, Arc<StackHawkClient>)> {
    let config = Config::load().ok()?;
    config.api_key.as_ref()?; // Require API key

    let client = Arc::new(StackHawkClient::new(config.api_key.clone()).ok()?);

    // Set JWT if cached and valid
    if !config.is_token_expired() {
        if let Some(ref jwt) = config.jwt {
            let rt = blocking_runtime()?;
            let client_clone = client.clone();
            let token = JwtToken {
                token: jwt.token.clone(),
                expires_at: jwt.expires_at,
            };
            rt.block_on(async move {
                client_clone.set_jwt(token).await;
            });
        }
    } else {
        // Need to authenticate - do it with timeout
        let rt = blocking_runtime()?;
        let client_clone = client.clone();
        let api_key = config.api_key.clone()?;
        let result = rt.block_on(async move {
            tokio::time::timeout(COMPLETION_TIMEOUT, client_clone.authenticate(&api_key)).await
        });

        match result {
            Ok(Ok(jwt)) => {
                let rt = blocking_runtime()?;
                let client_clone = client.clone();
                rt.block_on(async move {
                    client_clone.set_jwt(jwt).await;
                });
            }
            _ => return None, // Auth failed or timed out
        }
    }

    Some((config, client))
}

/// Complete scan IDs with rich metadata.
///
/// Format: `{scan_id}` with help `{app} | {env} | {status} | {date}`
///
/// Note: clap_complete handles prefix filtering - we return all candidates.
pub fn complete_scan_ids() -> Vec<CompletionCandidate> {
    let Some((config, client)) = completion_context() else {
        return vec![];
    };

    let Some(org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    let Some(rt) = blocking_runtime() else {
        return vec![];
    };

    let pagination = PaginationParams::new().page_size(MAX_COMPLETIONS);

    let result = rt.block_on(async {
        tokio::time::timeout(
            COMPLETION_TIMEOUT,
            client.list_scans(org_id, Some(&pagination), None),
        )
        .await
    });

    let scans = match result {
        Ok(Ok(scans)) => scans,
        _ => return vec![],
    };

    scans
        .into_iter()
        .map(|scan_result| {
            let scan = &scan_result.scan;
            let help = format!(
                "{} | {} | {} | {}",
                if scan.application_name.is_empty() {
                    "--"
                } else {
                    &scan.application_name
                },
                if scan.env.is_empty() { "--" } else { &scan.env },
                if scan.status.is_empty() {
                    "--"
                } else {
                    &scan.status
                },
                format_timestamp(&scan.timestamp)
            );
            CompletionCandidate::new(scan.id.clone()).help(Some(help.into()))
        })
        .collect()
}

/// Complete application names with metadata.
///
/// Format: `{app_name}` with help `{env} | {status}`
///
/// Note: clap_complete handles prefix filtering - we return all candidates.
pub fn complete_app_names() -> Vec<CompletionCandidate> {
    let Some((config, client)) = completion_context() else {
        return vec![];
    };

    let Some(org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    let Some(rt) = blocking_runtime() else {
        return vec![];
    };

    let result = rt.block_on(async {
        tokio::time::timeout(COMPLETION_TIMEOUT, client.list_apps(org_id, None)).await
    });

    let apps = match result {
        Ok(Ok(apps)) => apps,
        _ => return vec![],
    };

    apps.into_iter()
        .take(MAX_COMPLETIONS)
        .map(|app| {
            let help = format!(
                "{} | {}",
                app.env.as_deref().unwrap_or("--"),
                app.status.as_deref().unwrap_or("--")
            );
            CompletionCandidate::new(app.name).help(Some(help.into()))
        })
        .collect()
}

/// Complete plugin IDs for a specific scan.
///
/// Requires scan ID to be present in the command line.
/// Format: `{plugin_id}` with help `{name} | {severity} | {count} paths`
///
/// Note: Context-dependent completions (needing scan_id from args) are complex
/// with clap_complete's current API. For now, returns empty.
/// Future: Parse env vars or use clap's completion context when available.
pub fn complete_plugin_ids() -> Vec<CompletionCandidate> {
    // TODO: Extract scan_id from command line context
    // clap_complete's ValueCandidates doesn't receive parsed args,
    // so context-dependent completions require workarounds
    vec![]
}

/// Complete URI IDs for a specific scan.
///
/// Requires scan ID to be present in the command line.
/// Format: `{uri_id}` with help `{method} {path} | {plugin} | {severity}`
///
/// Note: Context-dependent completions (needing scan_id from args) are complex
/// with clap_complete's current API. For now, returns empty.
pub fn complete_uri_ids() -> Vec<CompletionCandidate> {
    // TODO: Extract scan_id from command line context
    vec![]
}

/// Format a Unix timestamp to short format (Jan 3 20:54)
fn format_timestamp(timestamp: &str) -> String {
    // Timestamp is Unix epoch in milliseconds (as string)
    timestamp
        .parse::<i64>()
        .ok()
        .and_then(|ts| {
            // API returns milliseconds, convert to seconds
            let secs = if ts > 1_000_000_000_000 {
                ts / 1000
            } else {
                ts
            };
            chrono::DateTime::from_timestamp(secs, 0)
        })
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%b %-d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "--".to_string())
}

/// Create completion candidates for scan IDs.
pub fn scan_id_candidates() -> ArgValueCandidates {
    ArgValueCandidates::new(complete_scan_ids)
}

/// Create completion candidates for app names.
pub fn app_name_candidates() -> ArgValueCandidates {
    ArgValueCandidates::new(complete_app_names)
}

/// Create completion candidates for plugin IDs.
pub fn plugin_id_candidates() -> ArgValueCandidates {
    ArgValueCandidates::new(complete_plugin_ids)
}

/// Create completion candidates for URI IDs.
pub fn uri_id_candidates() -> ArgValueCandidates {
    ArgValueCandidates::new(complete_uri_ids)
}
