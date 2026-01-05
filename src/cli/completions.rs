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
/// URI completion needs more time due to multiple parallel requests.
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

/// Extract scan ID from command line args during completion.
///
/// During completion, the shell passes the partial command line as args.
/// For `hawkop scan get <scan_id> --plugin-id <TAB>`, we need to find
/// the scan ID in position after "scan get".
fn extract_scan_id_from_args() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    // Find "scan" followed by "get" (or "g" alias) in args
    let mut found_scan = false;
    let mut found_get = false;

    for arg in args.iter() {
        if arg == "scan" {
            found_scan = true;
            continue;
        }
        if found_scan && (arg == "get" || arg == "g") {
            found_get = true;
            continue;
        }
        if found_get {
            // Next non-flag argument after "scan get" is the scan ID
            if !arg.starts_with('-') && !arg.is_empty() && arg != "latest" {
                // Validate it looks like a UUID (has dashes)
                if arg.contains('-') && arg.len() > 8 {
                    return Some(arg.clone());
                }
            }
            // If we hit a flag, keep looking in case scan_id comes later
            if arg.starts_with('-') {
                continue;
            }
        }
    }
    None
}

/// Complete plugin IDs for a specific scan.
///
/// Parses command line to extract scan ID, then fetches alerts for that scan.
/// Format: `{plugin_id}` with help `{name} | {severity} | {count} paths`
pub fn complete_plugin_ids() -> Vec<CompletionCandidate> {
    // Extract scan ID from command line context
    let Some(scan_id) = extract_scan_id_from_args() else {
        return vec![];
    };

    let Some((config, client)) = completion_context() else {
        return vec![];
    };

    let Some(_org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    let Some(rt) = blocking_runtime() else {
        return vec![];
    };

    let result = rt.block_on(async {
        tokio::time::timeout(COMPLETION_TIMEOUT, client.list_scan_alerts(&scan_id, None)).await
    });

    let alerts = match result {
        Ok(Ok(alerts)) => alerts,
        _ => return vec![],
    };

    alerts
        .into_iter()
        .take(MAX_COMPLETIONS)
        .map(|alert| {
            let path_word = if alert.uri_count == 1 {
                "path"
            } else {
                "paths"
            };
            let help = format!(
                "{} | {} | {} {}",
                truncate_str(&alert.name, 30),
                alert.severity,
                alert.uri_count,
                path_word
            );
            CompletionCandidate::new(alert.plugin_id).help(Some(help.into()))
        })
        .collect()
}

/// Complete URI IDs for a specific scan.
///
/// Parses command line to extract scan ID and optionally plugin ID,
/// then fetches URIs for that scan/plugin.
/// Format: `{uri_id}` with help `{method} {path} | {plugin_name}`
///
/// If plugin ID is specified, fetches URIs for that plugin only (fast).
/// Otherwise, fetches all plugins and their URIs in parallel (slower but complete).
pub fn complete_uri_ids() -> Vec<CompletionCandidate> {
    // Extract scan ID from command line context
    let Some(scan_id) = extract_scan_id_from_args() else {
        return vec![];
    };

    // Extract plugin ID if present (for scoped URI completion)
    let plugin_id = extract_plugin_id_from_args();

    let Some((config, client)) = completion_context() else {
        return vec![];
    };

    let Some(_org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    let Some(rt) = blocking_runtime() else {
        return vec![];
    };

    // If plugin_id is specified, fetch URIs for that plugin only (fast path)
    if let Some(ref pid) = plugin_id {
        let result = rt.block_on(async {
            tokio::time::timeout(
                COMPLETION_TIMEOUT,
                client.get_alert_with_paths(&scan_id, pid, None),
            )
            .await
        });

        let alert_response = match result {
            Ok(Ok(response)) => response,
            _ => return vec![],
        };

        let alert_name = alert_response.alert.name.clone();
        return alert_response
            .application_scan_alert_uris
            .into_iter()
            .take(MAX_COMPLETIONS)
            .map(|uri| {
                let help = format!(
                    "{} {} | {}",
                    uri.request_method,
                    truncate_str(&uri.uri, 40),
                    &alert_name
                );
                CompletionCandidate::new(uri.alert_uri_id).help(Some(help.into()))
            })
            .collect();
    }

    // No plugin_id specified - fetch all plugins and their URIs in parallel
    // This is slower but provides complete URI completion
    rt.block_on(async { complete_all_uri_ids(&scan_id, client).await })
}

/// Fetch all URI IDs for a scan by querying each plugin in parallel.
///
/// This is used when --plugin-id is not specified. It:
/// 1. Fetches all alerts (plugins) for the scan
/// 2. Fetches URIs for each plugin in parallel
/// 3. Collects up to MAX_COMPLETIONS URIs total
async fn complete_all_uri_ids(
    scan_id: &str,
    client: Arc<StackHawkClient>,
) -> Vec<CompletionCandidate> {
    use futures::future::join_all;

    // First, get all alerts (plugins) for this scan
    let alerts_result =
        tokio::time::timeout(COMPLETION_TIMEOUT, client.list_scan_alerts(scan_id, None)).await;

    let alerts = match alerts_result {
        Ok(Ok(alerts)) => alerts,
        _ => return vec![],
    };

    if alerts.is_empty() {
        return vec![];
    }

    // Fetch URIs for each plugin in parallel
    // Limit to first 10 plugins to avoid too many API calls
    let plugin_futures: Vec<_> = alerts
        .iter()
        .take(10)
        .map(|alert| {
            let client = client.clone();
            let scan_id = scan_id.to_string();
            let plugin_id = alert.plugin_id.clone();
            let plugin_name = alert.name.clone();

            async move {
                let result = tokio::time::timeout(
                    COMPLETION_TIMEOUT,
                    client.get_alert_with_paths(&scan_id, &plugin_id, None),
                )
                .await;

                match result {
                    Ok(Ok(response)) => response
                        .application_scan_alert_uris
                        .into_iter()
                        .map(|uri| (uri, plugin_name.clone()))
                        .collect::<Vec<_>>(),
                    _ => vec![],
                }
            }
        })
        .collect();

    // Apply overall timeout for all parallel requests
    let all_uris_result = tokio::time::timeout(
        COMPLETION_TIMEOUT * 2, // Give extra time for parallel requests
        join_all(plugin_futures),
    )
    .await;

    let all_uris = match all_uris_result {
        Ok(results) => results.into_iter().flatten().collect::<Vec<_>>(),
        _ => return vec![],
    };

    // Convert to completion candidates
    all_uris
        .into_iter()
        .take(MAX_COMPLETIONS)
        .map(|(uri, plugin_name)| {
            let help = format!(
                "{} {} | {}",
                uri.request_method,
                truncate_str(&uri.uri, 40),
                truncate_str(&plugin_name, 25)
            );
            CompletionCandidate::new(uri.alert_uri_id).help(Some(help.into()))
        })
        .collect()
}

/// Extract plugin ID from command line args during completion.
fn extract_plugin_id_from_args() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    for (i, arg) in args.iter().enumerate() {
        if (arg == "--plugin-id" || arg == "-p") && i + 1 < args.len() {
            let value = &args[i + 1];
            if !value.is_empty() && !value.starts_with('-') {
                return Some(value.clone());
            }
        }
    }
    None
}

/// Truncate a string to max length, adding "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
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
