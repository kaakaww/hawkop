//! Dynamic shell completions for HawkOp CLI
//!
//! Provides TAB completion for scan IDs, app names, plugin IDs, and URI IDs
//! by querying the StackHawk API at completion time. Results are cached locally
//! to improve responsiveness.
//!
//! Shell support:
//! - Fish/Zsh: Full support with descriptions
//! - Bash: Values only (no description display)

use std::sync::Arc;
use std::time::Duration;

use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::{CacheStorage, CacheTtl, cache_key};
use crate::client::models::JwtToken;
use crate::client::{PaginationParams, StackHawkApi, StackHawkClient};
use crate::config::Config;

/// Maximum number of completion candidates to return
const MAX_COMPLETIONS: usize = 10;

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

/// Get or create cache storage for completions.
/// Returns None if cache cannot be opened (completions will work without caching).
fn completion_cache() -> Option<CacheStorage> {
    CacheStorage::open().ok()
}

/// Try to get cached completion data.
fn get_cached<T: serde::de::DeserializeOwned>(cache: &CacheStorage, key: &str) -> Option<T> {
    cache
        .get(key)
        .ok()
        .flatten()
        .and_then(|data| serde_json::from_slice(&data).ok())
}

/// Store completion data in cache.
fn set_cached<T: serde::Serialize>(
    cache: &CacheStorage,
    key: &str,
    data: &T,
    endpoint: &str,
    org_id: Option<&str>,
    ttl: Duration,
) {
    if let Ok(json) = serde_json::to_vec(data) {
        let _ = cache.put(key, &json, endpoint, org_id, ttl);
    }
}

/// Complete scan IDs with rich metadata.
///
/// Format: `{scan_id}` with help `{app} | {env} | {status} | {date}`
///
/// Note: clap_complete handles prefix filtering - we return all candidates.
pub fn complete_scan_ids() -> Vec<CompletionCandidate> {
    let config = match Config::load().ok() {
        Some(c) => c,
        None => return vec![],
    };

    let Some(org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    // Try cache first
    let cache = completion_cache();
    let cache_key = cache_key("complete_scan_ids", Some(org_id), &[]);

    if let Some(ref c) = cache {
        let cached: Option<Vec<(String, String)>> = get_cached(c, &cache_key);
        if let Some(data) = cached {
            return data
                .into_iter()
                .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
                .collect();
        }
    }

    // Cache miss - need full context for API call
    let Some((_, client)) = completion_context() else {
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

    // Build completion data and cache it
    let completion_data: Vec<(String, String)> = scans
        .iter()
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
            (scan.id.clone(), help)
        })
        .collect();

    // Cache the results
    if let Some(ref c) = cache {
        set_cached(
            c,
            &cache_key,
            &completion_data,
            "scans",
            Some(org_id),
            CacheTtl::SCAN_LIST,
        );
    }

    completion_data
        .into_iter()
        .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
        .collect()
}

/// Complete application names with metadata.
///
/// Format: `{app_name}` with help `{env} | {status}`
///
/// Note: clap_complete handles prefix filtering - we return all candidates.
pub fn complete_app_names() -> Vec<CompletionCandidate> {
    let config = match Config::load().ok() {
        Some(c) => c,
        None => return vec![],
    };

    let Some(org_id) = config.org_id.as_ref() else {
        return vec![];
    };

    // Try cache first
    let cache = completion_cache();
    let cache_key = cache_key("complete_app_names", Some(org_id), &[]);

    if let Some(ref c) = cache {
        let cached: Option<Vec<(String, String)>> = get_cached(c, &cache_key);
        if let Some(data) = cached {
            return data
                .into_iter()
                .map(|(name, help)| CompletionCandidate::new(name).help(Some(help.into())))
                .collect();
        }
    }

    // Cache miss - need full context for API call
    let Some((_, client)) = completion_context() else {
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

    // Build completion data and cache it
    let completion_data: Vec<(String, String)> = apps
        .into_iter()
        .take(MAX_COMPLETIONS)
        .map(|app| {
            let help = format!(
                "{} | {}",
                app.env.as_deref().unwrap_or("--"),
                app.status.as_deref().unwrap_or("--")
            );
            (app.name, help)
        })
        .collect();

    // Cache the results
    if let Some(ref c) = cache {
        set_cached(
            c,
            &cache_key,
            &completion_data,
            "apps",
            Some(org_id),
            CacheTtl::APPS,
        );
    }

    completion_data
        .into_iter()
        .map(|(name, help)| CompletionCandidate::new(name).help(Some(help.into())))
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
/// Format: `{plugin_id}` with help `{severity} │ {name} │ {count} paths`
/// Sorted by severity (High → Medium → Low) and cached for 4 hours.
pub fn complete_plugin_ids() -> Vec<CompletionCandidate> {
    // Extract scan ID from command line context
    let Some(scan_id) = extract_scan_id_from_args() else {
        return vec![];
    };

    // Try cache first (keyed by scan_id)
    let cache = completion_cache();
    let cache_key = cache_key("complete_plugin_ids", None, &[("scan_id", &scan_id)]);

    if let Some(ref c) = cache {
        let cached: Option<Vec<(String, String)>> = get_cached(c, &cache_key);
        if let Some(data) = cached {
            return data
                .into_iter()
                .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
                .collect();
        }
    }

    // Cache miss - fetch from API
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

    let mut alerts = match result {
        Ok(Ok(alerts)) => alerts,
        _ => return vec![],
    };

    // Sort by severity: High → Medium → Low (most critical first)
    alerts.sort_by(|a, b| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)));

    // Build completion data with aligned columns
    let completion_data: Vec<(String, String)> = alerts
        .into_iter()
        .take(MAX_COMPLETIONS)
        .map(|alert| {
            let path_word = if alert.uri_count == 1 {
                "path"
            } else {
                "paths"
            };
            // Format: severity (padded) │ name │ count
            let help = format!(
                "{:6} │ {} │ {} {}",
                alert.severity,
                truncate_str(&alert.name, 32),
                alert.uri_count,
                path_word
            );
            (alert.plugin_id, help)
        })
        .collect();

    // Cache for 4 hours (scan findings are stable)
    if let Some(ref c) = cache {
        set_cached(
            c,
            &cache_key,
            &completion_data,
            "scan_alerts",
            None,
            CacheTtl::COMPLETION_ALERTS,
        );
    }

    completion_data
        .into_iter()
        .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
        .collect()
}

/// Rank severity for sorting (lower = more severe)
fn severity_rank(severity: &str) -> u8 {
    match severity {
        "High" => 0,
        "Medium" => 1,
        "Low" => 2,
        "Informational" | "Info" => 3,
        _ => 4,
    }
}

/// Complete URI IDs for a specific scan.
///
/// Parses command line to extract scan ID and optionally plugin ID,
/// then fetches URIs for that scan/plugin.
/// Format: `{uri_id}` with help `{severity} │ {method} {path} │ {plugin_name}`
///
/// If plugin ID is specified, fetches URIs for that plugin only (fast).
/// Otherwise, fetches all plugins and their URIs in parallel (slower but complete).
/// Sorted by severity (High → Medium → Low) and cached for 4 hours.
pub fn complete_uri_ids() -> Vec<CompletionCandidate> {
    // Extract scan ID from command line context
    let Some(scan_id) = extract_scan_id_from_args() else {
        return vec![];
    };

    // Extract plugin ID if present (for scoped URI completion)
    let plugin_id = extract_plugin_id_from_args();

    // Build cache key based on scan_id and optional plugin_id
    let cache = completion_cache();
    let cache_key = match &plugin_id {
        Some(pid) => cache_key(
            "complete_uri_ids",
            None,
            &[("scan_id", &scan_id), ("plugin_id", pid)],
        ),
        None => cache_key("complete_uri_ids", None, &[("scan_id", &scan_id)]),
    };

    // Try cache first
    if let Some(ref c) = cache {
        let cached: Option<Vec<(String, String)>> = get_cached(c, &cache_key);
        if let Some(data) = cached {
            return data
                .into_iter()
                .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
                .collect();
        }
    }

    // Cache miss - fetch from API
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
        let severity = alert_response.alert.severity.clone();

        let completion_data: Vec<(String, String)> = alert_response
            .application_scan_alert_uris
            .into_iter()
            .take(MAX_COMPLETIONS)
            .map(|uri| {
                let help = format!(
                    "{:6} │ {:4} {} │ {}",
                    severity,
                    uri.request_method,
                    truncate_str(&uri.uri, 35),
                    truncate_str(&alert_name, 25)
                );
                (uri.alert_uri_id, help)
            })
            .collect();

        // Cache for 4 hours
        if let Some(ref c) = cache {
            set_cached(
                c,
                &cache_key,
                &completion_data,
                "scan_uri_ids",
                None,
                CacheTtl::COMPLETION_ALERTS,
            );
        }

        return completion_data
            .into_iter()
            .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
            .collect();
    }

    // No plugin_id specified - fetch all plugins and their URIs in parallel
    let completion_data =
        rt.block_on(async { complete_all_uri_ids_cached(&scan_id, client).await });

    // Cache the all-plugins result for 4 hours
    if let Some(ref c) = cache {
        set_cached(
            c,
            &cache_key,
            &completion_data,
            "scan_uri_ids",
            None,
            CacheTtl::COMPLETION_ALERTS,
        );
    }

    completion_data
        .into_iter()
        .map(|(id, help)| CompletionCandidate::new(id).help(Some(help.into())))
        .collect()
}

/// Fetch all URI IDs for a scan by querying each plugin in parallel.
///
/// Returns cacheable `(uri_id, help_text)` tuples sorted by severity.
///
/// This is used when --plugin-id is not specified. It:
/// 1. Fetches all alerts (plugins) for the scan, sorted by severity
/// 2. Fetches URIs for each plugin using streaming futures
/// 3. Returns URIs sorted by their plugin's severity (High → Medium → Low)
async fn complete_all_uri_ids_cached(
    scan_id: &str,
    client: Arc<StackHawkClient>,
) -> Vec<(String, String)> {
    use futures::stream::{FuturesUnordered, StreamExt};

    // First, get all alerts (plugins) for this scan
    let alerts_result =
        tokio::time::timeout(COMPLETION_TIMEOUT, client.list_scan_alerts(scan_id, None)).await;

    let mut alerts = match alerts_result {
        Ok(Ok(alerts)) => alerts,
        _ => return vec![],
    };

    if alerts.is_empty() {
        return vec![];
    }

    // Sort alerts by severity so we fetch high-severity first
    alerts.sort_by(|a, b| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)));

    // Create streaming futures for each plugin (up to 10), preserving severity order
    let mut futures: FuturesUnordered<_> = alerts
        .iter()
        .take(10)
        .enumerate()
        .map(|(priority, alert)| {
            let client = client.clone();
            let scan_id = scan_id.to_string();
            let plugin_id = alert.plugin_id.clone();
            let plugin_name = alert.name.clone();
            let severity = alert.severity.clone();

            async move {
                let result = client
                    .get_alert_with_paths(&scan_id, &plugin_id, None)
                    .await;

                match result {
                    Ok(response) => response
                        .application_scan_alert_uris
                        .into_iter()
                        .map(|uri| (uri, plugin_name.clone(), severity.clone(), priority))
                        .collect::<Vec<_>>(),
                    _ => vec![],
                }
            }
        })
        .collect();

    let mut all_uris: Vec<(String, String, usize)> = Vec::new();

    // Process results as they complete
    let stream_with_timeout = async {
        while let Some(uris) = futures.next().await {
            for (uri, plugin_name, severity, priority) in uris {
                let help = format!(
                    "{:6} │ {:4} {} │ {}",
                    severity,
                    uri.request_method,
                    truncate_str(&uri.uri, 35),
                    truncate_str(&plugin_name, 25)
                );
                all_uris.push((uri.alert_uri_id, help, priority));
            }
        }
    };

    let _ = tokio::time::timeout(COMPLETION_TIMEOUT * 2, stream_with_timeout).await;

    // Sort by severity priority, then take top MAX_COMPLETIONS
    all_uris.sort_by_key(|(_, _, priority)| *priority);
    all_uris
        .into_iter()
        .take(MAX_COMPLETIONS)
        .map(|(id, help, _)| (id, help))
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
