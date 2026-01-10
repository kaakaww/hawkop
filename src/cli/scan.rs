//! Scan management commands

use chrono::{TimeZone, Utc};
use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
use crate::client::models::ScanResult;
use crate::client::{
    ListingApi, PaginationParams, ScanDetailApi, ScanFilterParams, fetch_remaining_pages,
};
use crate::error::Result;
use crate::models::{
    AlertDetail, AlertFindingDisplay, AlertMessageDetail, PrettyAlertDisplay, ScanDisplay,
};
use crate::output::Formattable;

// ============================================================================
// Scan Context for Banner Display
// ============================================================================

/// Context information for scan banner display
#[derive(Debug, Clone)]
pub struct ScanContext {
    /// Application name
    pub app_name: String,
    /// Environment name
    pub env: String,
    /// Application host URL
    pub host: Option<String>,
    /// Scan ID (UUID)
    pub scan_id: String,
    /// HawkScan version
    pub version: String,
    /// Scan timestamp (Unix epoch milliseconds)
    pub timestamp: String,
    /// Scan duration in seconds
    pub duration: Option<String>,
    /// Scan status
    pub status: String,
}

impl ScanContext {
    /// Create scan context from a ScanResult
    pub fn from_scan_result(scan: &ScanResult) -> Self {
        Self {
            app_name: scan.scan.application_name.clone(),
            env: scan.scan.env.clone(),
            host: scan.app_host.clone(),
            scan_id: scan.scan.id.clone(),
            version: scan.scan.version.clone(),
            timestamp: scan.scan.timestamp.clone(),
            duration: scan.scan_duration.clone(),
            status: scan.scan.status.clone(),
        }
    }

    /// Format the scan banner for display (consistent with default view)
    ///
    /// Produces a condensed banner matching the default view style:
    /// ```text
    /// App: X | Env: Y | Host: Z
    /// Scan ID: ... | Completed: ... | Duration: ... | Status: ...
    /// HawkScan: ...
    /// ```
    pub fn format_banner(&self) -> String {
        let mut lines = Vec::new();

        // Line 1: App | Env | Host
        let app_name = if self.app_name.is_empty() {
            "--".to_string()
        } else {
            self.app_name.clone()
        };
        let env_name = if self.env.is_empty() {
            "--".to_string()
        } else {
            self.env.clone()
        };
        if let Some(ref host) = self.host {
            if !host.is_empty() {
                lines.push(format!(
                    "App: {} | Env: {} | Host: {}",
                    app_name, env_name, host
                ));
            } else {
                lines.push(format!("App: {} | Env: {}", app_name, env_name));
            }
        } else {
            lines.push(format!("App: {} | Env: {}", app_name, env_name));
        }

        // Line 2: Scan ID | Completed | Duration | Status
        let date_str = format_timestamp_local(&self.timestamp);
        let duration_str = self
            .duration
            .as_ref()
            .map(|d| format_duration_seconds(d))
            .unwrap_or_else(|| "--".to_string());
        let status_str = format_scan_status(&self.status);
        lines.push(format!(
            "Scan ID: {} | Completed: {} | Duration: {} | Status: {}",
            self.scan_id, date_str, duration_str, status_str
        ));

        // Line 3: HawkScan version
        let version_str = if self.version.is_empty() {
            "--".to_string()
        } else {
            self.version.clone()
        };
        lines.push(format!("HawkScan: {}", version_str));

        lines.join("\n")
    }
}

/// Format Unix timestamp (milliseconds) to local date/time string
fn format_timestamp_local(timestamp: &str) -> String {
    let millis: i64 = timestamp.parse().unwrap_or(0);
    if millis == 0 {
        return "N/A".to_string();
    }

    let secs = millis / 1000;
    match Utc.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt) => {
            let local = dt.with_timezone(&chrono::Local);
            let date_time = local.format("%m/%d/%Y %H:%M").to_string();
            let tz_abbrev = offset_to_tz_abbrev(local.offset().local_minus_utc());
            format!("{} {}", date_time, tz_abbrev)
        }
        _ => "N/A".to_string(),
    }
}

/// Convert UTC offset (seconds) to timezone abbreviation
fn offset_to_tz_abbrev(offset_secs: i32) -> &'static str {
    let offset_hours = offset_secs / 3600;
    match offset_hours {
        -12 => "IDLW",
        -11 => "SST",
        -10 => "HST",
        -9 => "AKST",
        -8 => "PST",
        -7 => "MST",
        -6 => "CST",
        -5 => "EST",
        -4 => "AST",
        -3 => "ART",
        0 => "UTC",
        1 => "CET",
        2 => "EET",
        3 => "MSK",
        5 | 6 => "IST",
        8 => "CST", // China Standard Time
        9 => "JST",
        10 => "AEST",
        12 => "NZST",
        _ => {
            // Fall back to offset format for uncommon zones
            // This is a static str leak but only happens for rare offsets
            Box::leak(format!("UTC{:+}", offset_hours).into_boxed_str())
        }
    }
}

/// Format duration in seconds to human-readable string
fn format_duration_seconds(seconds_str: &str) -> String {
    let secs: u64 = seconds_str.parse().unwrap_or(0);
    if secs == 0 {
        return "N/A".to_string();
    }

    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Default limit for scan list display
const DEFAULT_SCAN_LIMIT: usize = 10;

/// Requested page size for scans endpoint
const SCAN_API_PAGE_SIZE: usize = 100;

/// Max scans to fetch when sorting (to avoid runaway queries)
const MAX_SORT_FETCH: usize = 10_000;

/// Max concurrent requests in the worker pool
const PARALLEL_FETCH_LIMIT: usize = 32;

/// Run the scan list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    filters: &ScanFilterArgs,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
    let org_id = ctx.require_org_id()?;

    let display_limit = pagination.limit.unwrap_or(DEFAULT_SCAN_LIMIT);

    // Determine how many scans to fetch:
    // - Sorting requires all data (API doesn't support useful sort fields)
    // - Status filtering needs extra since filter ratio is unknown
    // - Otherwise just fetch what we need to display
    let has_sort = pagination.sort_by.is_some();
    let has_status_filter = filters.status.is_some();
    let target_count = if has_sort {
        MAX_SORT_FETCH // Fetch all available for accurate sorting
    } else if has_status_filter {
        display_limit * 10
    } else {
        display_limit
    };

    // Build server-side filter params for app and env
    let filter_params = if !filters.app.is_empty() || !filters.env.is_empty() {
        Some(
            ScanFilterParams::new()
                .app_ids(filters.app.clone())
                .envs(filters.env.clone()),
        )
    } else {
        None
    };

    // Fetch scans using totalCount-based parallel pagination
    // 1. First request gets totalCount
    // 2. Calculate remaining pages
    // 3. Fetch remaining pages in parallel
    let start_page = pagination.page.unwrap_or(0);
    let first_params = PaginationParams::new()
        .page_size(SCAN_API_PAGE_SIZE)
        .page(start_page);

    debug!(
        "Fetching first page to get totalCount (page={}, pageSize={})",
        start_page, SCAN_API_PAGE_SIZE
    );

    let first_response = ctx
        .client
        .list_scans_paged(org_id, Some(&first_params), filter_params.as_ref())
        .await?;

    let mut all_scans = first_response.items;
    debug!(
        "First page returned {} items, totalCount={:?}",
        all_scans.len(),
        first_response.total_count
    );

    // Calculate remaining pages based on totalCount and target
    if let Some(total_count) = first_response.total_count {
        // Limit to target_count to avoid fetching more than needed
        let effective_total = total_count.min(target_count);
        let total_pages = effective_total.div_ceil(SCAN_API_PAGE_SIZE);

        if total_pages > 1 {
            let remaining_pages: Vec<usize> = (start_page + 1..start_page + total_pages).collect();

            if !remaining_pages.is_empty() {
                debug!(
                    "Fetching {} remaining pages in parallel (totalCount={}, target={})",
                    remaining_pages.len(),
                    total_count,
                    target_count
                );

                let client = ctx.client.clone();
                let org = org_id.to_string();
                let filters_clone = filter_params.clone();

                let remaining_scans = fetch_remaining_pages(
                    remaining_pages,
                    move |page| {
                        let c = client.clone();
                        let o = org.clone();
                        let f = filters_clone.clone();
                        async move {
                            let params = PaginationParams::new()
                                .page_size(SCAN_API_PAGE_SIZE)
                                .page(page);
                            c.list_scans(&o, Some(&params), f.as_ref()).await
                        }
                    },
                    PARALLEL_FETCH_LIMIT,
                )
                .await?;

                all_scans.extend(remaining_scans);
            }
        }
    } else {
        // Fallback: no totalCount available, fetch until we have enough
        debug!("No totalCount available, fetching pages until target reached");
        let mut page = start_page + 1;
        while all_scans.len() < target_count {
            let params = PaginationParams::new()
                .page_size(SCAN_API_PAGE_SIZE)
                .page(page);
            let scans = ctx
                .client
                .list_scans(org_id, Some(&params), filter_params.as_ref())
                .await?;

            if scans.is_empty() {
                break;
            }
            all_scans.extend(scans);
            page += 1;
        }
    }

    debug!("Total scans fetched: {}", all_scans.len());

    // Apply client-side filtering for status (not supported server-side)
    let filtered_scans = apply_status_filter(all_scans, filters);

    // Apply client-side sorting (API doesn't support useful sort fields)
    let sorted_scans = apply_sort(filtered_scans, pagination);

    // Apply display limit
    let limited_scans: Vec<_> = sorted_scans.into_iter().take(display_limit).collect();

    // Convert to display models
    let display_scans: Vec<ScanDisplay> =
        limited_scans.into_iter().map(ScanDisplay::from).collect();
    display_scans.print(ctx.format)?;

    Ok(())
}

/// Check if a scan matches the status filter.
fn matches_status(scan: &ScanResult, status_filter: &str) -> bool {
    let status_lower = status_filter.to_lowercase();
    let status_upper = scan.scan.status.to_uppercase();
    let scan_status = match status_upper.as_str() {
        "STARTED" => "running",
        "COMPLETED" => "complete",
        "ERROR" => "failed",
        _ => &status_upper,
    };
    scan_status.to_lowercase().contains(&status_lower)
}

/// Apply client-side status filter to scan results.
/// Status filtering is not supported server-side, so we filter here.
fn apply_status_filter(scans: Vec<ScanResult>, filters: &ScanFilterArgs) -> Vec<ScanResult> {
    let Some(ref status_filter) = filters.status else {
        return scans;
    };
    scans
        .into_iter()
        .filter(|scan| matches_status(scan, status_filter))
        .collect()
}

/// Apply client-side sorting to scan results.
/// API has limited sort field support, so we sort client-side for better UX.
fn apply_sort(mut scans: Vec<ScanResult>, pagination: &PaginationArgs) -> Vec<ScanResult> {
    let Some(ref sort_by) = pagination.sort_by else {
        return scans;
    };

    let descending = matches!(pagination.sort_dir, Some(SortDir::Desc));
    let sort_by_lower = sort_by.to_lowercase();

    scans.sort_by(|a, b| {
        let cmp = match sort_by_lower.as_str() {
            "app" | "application" | "appname" | "applicationname" => a
                .scan
                .application_name
                .to_lowercase()
                .cmp(&b.scan.application_name.to_lowercase()),
            "env" | "environment" => a.scan.env.to_lowercase().cmp(&b.scan.env.to_lowercase()),
            "status" => a
                .scan
                .status
                .to_lowercase()
                .cmp(&b.scan.status.to_lowercase()),
            "started" | "timestamp" | "time" | "date" => a.scan.timestamp.cmp(&b.scan.timestamp),
            "duration" => {
                let a_dur: f64 = a
                    .scan_duration
                    .as_deref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                let b_dur: f64 = b
                    .scan_duration
                    .as_deref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                a_dur
                    .partial_cmp(&b_dur)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
            "findings" | "alerts" => {
                // Sort by new findings: High first, then Medium, then Low
                let a_findings = get_new_findings(a);
                let b_findings = get_new_findings(b);
                a_findings.cmp(&b_findings)
            }
            "id" => a.scan.id.cmp(&b.scan.id),
            _ => std::cmp::Ordering::Equal,
        };

        if descending { cmp.reverse() } else { cmp }
    });

    scans
}

/// Extract new findings counts as (high, medium, low) tuple for sorting.
/// Returns counts in priority order so tuple comparison works correctly.
fn get_new_findings(scan: &ScanResult) -> (u32, u32, u32) {
    let Some(ref alert_stats) = scan.alert_stats else {
        return (0, 0, 0);
    };

    // Find the "UNKNOWN" status which represents new/untriaged findings
    let new_stats = alert_stats
        .alert_status_stats
        .iter()
        .find(|s| s.alert_status == "UNKNOWN");

    let Some(stats) = new_stats else {
        return (0, 0, 0);
    };

    let high = *stats.severity_stats.get("High").unwrap_or(&0);
    let medium = *stats.severity_stats.get("Medium").unwrap_or(&0);
    let low = *stats.severity_stats.get("Low").unwrap_or(&0);

    (high, medium, low)
}

// ============================================================================
// Scan Get / Drill-Down Commands
// ============================================================================

/// Run the scan get command with flag-based drill-down
///
/// Supports flag-based navigation for exploring scan results:
/// - `scan get` - Latest scan with overview + alerts table
/// - `scan get <id>` - Specific scan overview + alerts
/// - `scan get <id> --plugin-id <p>` - Plugin detail with paths
/// - `scan get <id> --uri-id <u>` - URI detail with evidence
/// - `scan get <id> --uri-id <u> -m` - URI detail with HTTP message
#[allow(clippy::too_many_arguments)]
pub async fn get(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    scan_id: &str,
    app: Option<&str>,
    app_id: Option<&str>,
    env: Option<&str>,
    plugin_id: Option<&str>,
    uri_id: Option<&str>,
    message: bool,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
    let org_id = ctx.require_org_id()?;

    // Validate: can't use filters with specific scan ID
    let is_latest = scan_id == "latest" || scan_id.is_empty();
    if !is_latest && (app.is_some() || app_id.is_some() || env.is_some()) {
        return Err(crate::error::ApiError::BadRequest(
            "Cannot specify both scan ID and filters (--app, --app-id, --env). \
             Use filters only with 'latest' or omit scan ID."
                .to_string(),
        )
        .into());
    }

    // Resolve scan ID if using "latest"
    let resolved_id = if is_latest {
        resolve_latest_scan(&ctx, org_id, app, app_id, env).await?
    } else {
        scan_id.to_string()
    };

    debug!(
        "Scan get: id={}, plugin={:?}, uri={:?}, message={}",
        resolved_id, plugin_id, uri_id, message
    );

    // Determine detail level based on flags
    match (plugin_id, uri_id, message) {
        (None, None, false) => show_pretty_overview(&ctx, org_id, &resolved_id).await,
        (Some(p), None, false) => show_alert_detail(&ctx, org_id, &resolved_id, p).await,
        (_, Some(u), false) => show_uri_detail_by_id(&ctx, org_id, &resolved_id, u).await,
        (_, Some(u), true) => show_message_by_uri(&ctx, org_id, &resolved_id, u).await,
        _ => Err(crate::error::ApiError::BadRequest(
            "Invalid flag combination. Use --uri-id to show finding detail, add -m for HTTP message."
                .to_string(),
        )
        .into()),
    }
}

/// Resolve "latest" scan ID with optional app/env filters
///
/// Supports two ways to filter by application:
/// - `app`: Filter by application name (looks up the app ID via API)
/// - `app_id`: Filter by application ID directly
async fn resolve_latest_scan(
    ctx: &CommandContext,
    org_id: &str,
    app: Option<&str>,
    app_id: Option<&str>,
    env: Option<&str>,
) -> Result<String> {
    debug!(
        "Resolving latest scan (app={:?}, app_id={:?}, env={:?})",
        app, app_id, env
    );

    // Resolve app name to app ID if provided
    let resolved_app_id = if let Some(app_name) = app {
        // Look up application by name (handle duplicates)
        let apps = ctx.client.list_apps(org_id, None).await?;
        let matching_apps: Vec<_> = apps
            .iter()
            .filter(|a| a.name.eq_ignore_ascii_case(app_name))
            .collect();

        match matching_apps.len() {
            0 => {
                return Err(crate::error::ApiError::NotFound(format!(
                    "Application '{}' not found. Use 'hawkop app list' to see available applications.",
                    app_name
                ))
                .into());
            }
            1 => {
                debug!(
                    "Resolved app '{}' to ID '{}'",
                    app_name, matching_apps[0].id
                );
                Some(matching_apps[0].id.clone())
            }
            _ => {
                // Multiple apps with same name - require --app-id for disambiguation
                let mut msg = format!("Multiple applications match '{}':\n", app_name);
                for app in &matching_apps {
                    let env_info = app.env.as_deref().unwrap_or("--");
                    let short_id = &app.id[..8.min(app.id.len())];
                    msg.push_str(&format!(
                        "  • {} ({}) - env: {}\n",
                        app.name, short_id, env_info
                    ));
                }
                msg.push_str("\nUse --app-id <uuid> to specify exactly which one.");
                return Err(crate::error::ApiError::BadRequest(msg).into());
            }
        }
    } else {
        app_id.map(|s| s.to_string())
    };

    // Build filter params if any filters specified
    let filter_params = if resolved_app_id.is_some() || env.is_some() {
        let mut params = ScanFilterParams::new();
        if let Some(ref aid) = resolved_app_id {
            params = params.app_ids(vec![aid.clone()]);
        }
        if let Some(env_name) = env {
            params = params.envs(vec![env_name.to_string()]);
        }
        Some(params)
    } else {
        None
    };

    // Fetch just one scan (the most recent)
    let pagination = PaginationParams::new().page_size(1).page(0);
    let scans = ctx
        .client
        .list_scans(org_id, Some(&pagination), filter_params.as_ref())
        .await?;

    if scans.is_empty() {
        let filter_desc = match (app, app_id, env) {
            (Some(a), _, Some(e)) => format!(" for app '{}' and env '{}'", a, e),
            (Some(a), _, None) => format!(" for app '{}'", a),
            (_, Some(a), Some(e)) => format!(" for app ID '{}' and env '{}'", a, e),
            (_, Some(a), None) => format!(" for app ID '{}'", a),
            (None, None, Some(e)) => format!(" for env '{}'", e),
            (None, None, None) => String::new(),
        };
        return Err(crate::error::ApiError::NotFound(format!(
            "No scans found{}. Run a scan first or check your filters.",
            filter_desc
        ))
        .into());
    }

    let scan_id = scans[0].scan.id.clone();
    debug!("Resolved latest scan: {}", scan_id);
    Ok(scan_id)
}

/// Show pretty overview combining scan metadata + alerts table (default view)
///
/// Output format matches the mockup:
/// ```text
/// Scan ID: <uuid> | User: <email>
/// Completed: <date> | Duration: <duration>
/// HawkScan: <version> | Policy: <policy>
/// New: X High, Y Medium, Z Low | Triaged: A High, B Medium, C Low
///
/// PLUGIN  SEVERITY  NAME  PATHS  NEW  ASSIGNED  ACCEPTED  FALSE+  CWE
/// ...
///
/// Tags:
///   Key: Value
///
/// Continue: hawkop scan get <scan-id> --plugin-id <plugin-id>
/// ```
async fn show_pretty_overview(ctx: &CommandContext, org_id: &str, scan_id: &str) -> Result<()> {
    debug!("Fetching pretty overview for {}", scan_id);
    let scan = ctx.client.get_scan(org_id, scan_id).await?;

    match ctx.format {
        OutputFormat::Pretty => {
            // Fetch alerts for the table
            let alerts = ctx.client.list_scan_alerts(scan_id, None).await?;

            // Extract userId from metadata.tags (preferred) or fallback to scan.external_user_id
            let user_id = scan
                .metadata
                .as_ref()
                .and_then(|m| m.tags.get("userId").cloned())
                .or_else(|| scan.scan.external_user_id.clone())
                .filter(|id| !id.is_empty());

            // Look up user display name (email preferred, then username, then full name)
            let user_display = if let Some(ref uid) = user_id {
                lookup_user_display(ctx, org_id, uid).await
            } else {
                None
            };

            // Line 1: App | Env | Host (context line with labels)
            let app_name = if scan.scan.application_name.is_empty() {
                "--".to_string()
            } else {
                scan.scan.application_name.clone()
            };
            let env_name = if scan.scan.env.is_empty() {
                "--".to_string()
            } else {
                scan.scan.env.clone()
            };
            if let Some(ref host) = scan.app_host {
                if !host.is_empty() {
                    println!("App: {} | Env: {} | Host: {}", app_name, env_name, host);
                } else {
                    println!("App: {} | Env: {}", app_name, env_name);
                }
            } else {
                println!("App: {} | Env: {}", app_name, env_name);
            }

            // Line 2: Scan ID | User
            if let Some(ref user) = user_display {
                println!("Scan ID: {} | User: {}", scan.scan.id, user);
            } else {
                println!("Scan ID: {}", scan.scan.id);
            }

            // Line 3: Completed date | Duration | Status
            let completed_date = format_timestamp_local(&scan.scan.timestamp);
            let duration_str = scan
                .scan_duration
                .as_ref()
                .map(|d| format_duration_seconds(d))
                .unwrap_or_else(|| "--".to_string());
            let status_str = format_scan_status(&scan.scan.status);
            println!(
                "Completed: {} | Duration: {} | Status: {}",
                completed_date, duration_str, status_str
            );

            // Line 3: HawkScan version | Policy
            // Extract policy from metadata.tags.policyDisplayName (preferred) or fallback
            let policy_name = scan
                .metadata
                .as_ref()
                .and_then(|m| m.tags.get("policyDisplayName").cloned())
                .or_else(|| {
                    scan.metadata
                        .as_ref()
                        .and_then(|m| m.tags.get("policyName").cloned())
                })
                .or_else(|| scan.policy_name.clone())
                .filter(|p| !p.is_empty());

            let version_str = if scan.scan.version.is_empty() {
                "--".to_string()
            } else {
                scan.scan.version.clone()
            };
            // Only show policy if it's present AND non-empty
            if let Some(ref policy) = policy_name {
                println!("HawkScan: {} | Policy: {}", version_str, policy);
            } else {
                println!("HawkScan: {}", version_str);
            }

            // Line 4: Findings summary - New vs Triaged by severity
            let (new_summary, triaged_summary) = format_findings_summary(&scan);
            println!("New: {} | Triaged: {}", new_summary, triaged_summary);

            // Alerts table with detailed triage columns
            if !alerts.is_empty() {
                println!();
                // Sort by severity (High → Medium → Low) then by plugin_id for stable ordering
                let mut sorted_alerts = alerts;
                sorted_alerts.sort_by(|a, b| {
                    let severity_order = |s: &str| match s.to_uppercase().as_str() {
                        "HIGH" => 0,
                        "MEDIUM" => 1,
                        "LOW" => 2,
                        _ => 3,
                    };
                    severity_order(&a.severity)
                        .cmp(&severity_order(&b.severity))
                        .then_with(|| a.plugin_id.cmp(&b.plugin_id))
                });
                let display_alerts: Vec<PrettyAlertDisplay> = sorted_alerts
                    .into_iter()
                    .map(PrettyAlertDisplay::from)
                    .collect();
                display_alerts.print(OutputFormat::Table)?;
            } else {
                println!("\nNo findings.");
            }

            // Tags section - deduplicated and filtered
            if !scan.tags.is_empty() {
                // Deduplicate by tag name (keep first occurrence) and filter out:
                // - Unexpanded env vars like ${RELEASE_TAG}
                // - Empty values
                let mut seen = std::collections::HashSet::new();
                let filtered_tags: Vec<_> = scan
                    .tags
                    .iter()
                    .filter(|tag| {
                        // Skip duplicates
                        if !seen.insert(&tag.name) {
                            return false;
                        }
                        // Skip unexpanded env vars (contain ${...})
                        if tag.value.contains("${") && tag.value.contains('}') {
                            return false;
                        }
                        // Skip empty values
                        !tag.value.is_empty()
                    })
                    .collect();

                if !filtered_tags.is_empty() {
                    println!("\nTags:");
                    for tag in filtered_tags {
                        println!("  {}: {}", tag.name, tag.value);
                    }
                }
            }

            // Navigation hint (use full scan ID)
            eprintln!();
            eprintln!(
                "Continue: hawkop scan get {} --plugin-id <plugin-id>",
                scan_id
            );
        }
        OutputFormat::Table => {
            // Table format: just show scan as a single-row table
            let display_scans: Vec<ScanDisplay> = vec![ScanDisplay::from(scan)];
            display_scans.print(ctx.format)?;
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&scan)?;
            println!("{}", json);
        }
    }

    Ok(())
}

/// Look up user display name from Members API
///
/// Returns the best available identifier in order of preference:
/// 1. Email address (preferred - most identifiable)
/// 2. Full name
/// 3. First + Last name
async fn lookup_user_display(ctx: &CommandContext, org_id: &str, user_id: &str) -> Option<String> {
    debug!("Looking up user {} in org {}", user_id, org_id);

    // Fetch org members and find the one matching the user_id
    match ctx.client.list_users(org_id, None).await {
        Ok(users) => {
            let user = users.into_iter().find(|u| u.external.id == user_id)?;
            let ext = &user.external;

            // Prefer email, then full_name, then first+last
            if !ext.email.is_empty() {
                Some(ext.email.clone())
            } else if let Some(ref name) = ext.full_name {
                if !name.is_empty() {
                    return Some(name.clone());
                }
                None
            } else {
                // Construct from first + last
                match (&ext.first_name, &ext.last_name) {
                    (Some(f), Some(l)) if !f.is_empty() || !l.is_empty() => {
                        Some(format!("{} {}", f, l).trim().to_string())
                    }
                    (Some(f), None) if !f.is_empty() => Some(f.clone()),
                    (None, Some(l)) if !l.is_empty() => Some(l.clone()),
                    _ => None,
                }
            }
        }
        Err(e) => {
            debug!("Failed to lookup user {}: {}", user_id, e);
            None
        }
    }
}

/// Format findings summary as "X High, Y Medium, Z Low" for both new and triaged
fn format_findings_summary(result: &ScanResult) -> (String, String) {
    let alert_stats = match &result.alert_stats {
        Some(stats) => stats,
        None => return ("--".to_string(), "--".to_string()),
    };

    // Collect counts by severity and status
    let mut high_new = 0u32;
    let mut high_triaged = 0u32;
    let mut medium_new = 0u32;
    let mut medium_triaged = 0u32;
    let mut low_new = 0u32;
    let mut low_triaged = 0u32;

    for status_stat in &alert_stats.alert_status_stats {
        let is_new = status_stat.alert_status == "UNKNOWN";
        let is_triaged = status_stat.alert_status == "PROMOTED"
            || status_stat.alert_status == "ACCEPTED"
            || status_stat.alert_status == "FALSE_POSITIVE"
            || status_stat.alert_status == "RISK_ACCEPTED";

        for (severity, count) in &status_stat.severity_stats {
            match severity.as_str() {
                "High" => {
                    if is_new {
                        high_new += count;
                    } else if is_triaged {
                        high_triaged += count;
                    }
                }
                "Medium" => {
                    if is_new {
                        medium_new += count;
                    } else if is_triaged {
                        medium_triaged += count;
                    }
                }
                "Low" => {
                    if is_new {
                        low_new += count;
                    } else if is_triaged {
                        low_triaged += count;
                    }
                }
                _ => {}
            }
        }
    }

    let new_summary = format!("{} High, {} Medium, {} Low", high_new, medium_new, low_new);
    let triaged_summary = format!(
        "{} High, {} Medium, {} Low",
        high_triaged, medium_triaged, low_triaged
    );

    (new_summary, triaged_summary)
}

/// Show alert detail with paths (scan get <id> --plugin-id <plugin>)
async fn show_alert_detail(
    ctx: &CommandContext,
    org_id: &str,
    scan_id: &str,
    plugin_id: &str,
) -> Result<()> {
    debug!("Fetching alert {} for scan {}", plugin_id, scan_id);

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    let response = ctx
        .client
        .get_alert_with_paths(scan_id, plugin_id, None)
        .await?;

    match ctx.format {
        OutputFormat::Pretty | OutputFormat::Table => {
            // Display banner
            println!("{}\n", scan_context.format_banner());

            // Print header
            let detail = AlertDetail::new(response.clone());
            println!("{}", detail.format_header());

            // Print paths table
            let display_paths: Vec<AlertFindingDisplay> = response
                .application_scan_alert_uris
                .into_iter()
                .map(AlertFindingDisplay::from)
                .collect();

            display_paths.print(OutputFormat::Table)?;

            // Navigation hint (use full scan ID for consistency)
            if !display_paths.is_empty() {
                eprintln!();
                eprintln!("Continue: hawkop scan get {} --uri-id <uri-id>", scan_id);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&response)?;
            println!("{}", json);
        }
    }

    Ok(())
}

/// Show URI detail by URI ID (scan get <id> --uri-id <uri-id>)
///
/// Since URIs are unique within a scan, we can look up directly without plugin_id.
/// This searches all alerts in the scan to find the matching URI.
async fn show_uri_detail_by_id(
    ctx: &CommandContext,
    org_id: &str,
    scan_id: &str,
    uri_id: &str,
) -> Result<()> {
    debug!("Fetching URI detail for scan {} uri {}", scan_id, uri_id);

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    // Get all alerts to find the one containing this URI
    let alerts = ctx.client.list_scan_alerts(scan_id, None).await?;

    // Search each alert for the URI
    for alert in &alerts {
        let response = ctx
            .client
            .get_alert_with_paths(scan_id, &alert.plugin_id, None)
            .await?;

        if let Some(path) = response
            .application_scan_alert_uris
            .iter()
            .find(|p| p.alert_uri_id == uri_id)
        {
            // Found the URI - fetch message for evidence
            let message = ctx
                .client
                .get_alert_message(scan_id, uri_id, &path.msg_id, false)
                .await?;

            match ctx.format {
                OutputFormat::Pretty | OutputFormat::Table => {
                    // Display banner
                    println!("{}\n", scan_context.format_banner());

                    // Alert context
                    println!(
                        "{} [{}]",
                        response.alert.name,
                        format_severity(&response.alert.severity)
                    );
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );

                    // URI details
                    println!("Finding: {} {}", path.request_method, path.uri);
                    println!("Status:  {}", format_triage_status(&path.status));

                    // Evidence (if present)
                    if let Some(ref evidence) = message.evidence
                        && !evidence.is_empty()
                    {
                        println!("\nEvidence:");
                        println!("  {}", evidence);
                    }

                    // Other info (if present)
                    if let Some(ref other_info) = message.other_info
                        && !other_info.is_empty()
                    {
                        println!("\nOther Info:");
                        for line in other_info.lines() {
                            println!("  {}", line);
                        }
                    }

                    eprintln!();
                    eprintln!(
                        "Continue: hawkop scan get {} --uri-id {} -m",
                        scan_id, uri_id
                    );
                }
                OutputFormat::Json => {
                    let combined = serde_json::json!({
                        "uri": path,
                        "alert": {
                            "name": response.alert.name,
                            "severity": response.alert.severity,
                            "plugin_id": response.alert.plugin_id,
                        },
                        "evidence": message.evidence,
                        "other_info": message.other_info,
                    });
                    let json = serde_json::to_string_pretty(&combined)?;
                    println!("{}", json);
                }
            }

            return Ok(());
        }
    }

    Err(crate::error::ApiError::NotFound(format!(
        "URI '{}' not found in scan. Use 'hawkop scan get {} --plugin-id <id>' to see available URIs.",
        uri_id, scan_id
    ))
    .into())
}

/// Show HTTP message by URI ID (scan get <id> --uri-id <uri-id> -m)
async fn show_message_by_uri(
    ctx: &CommandContext,
    org_id: &str,
    scan_id: &str,
    uri_id: &str,
) -> Result<()> {
    debug!("Fetching message for scan {} uri {}", scan_id, uri_id);

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    // Get all alerts to find the one containing this URI
    let alerts = ctx.client.list_scan_alerts(scan_id, None).await?;

    // Search each alert for the URI
    for alert in &alerts {
        let response = ctx
            .client
            .get_alert_with_paths(scan_id, &alert.plugin_id, None)
            .await?;

        if let Some(path) = response
            .application_scan_alert_uris
            .iter()
            .find(|p| p.alert_uri_id == uri_id)
        {
            // Found the URI - fetch message with curl command
            let message = ctx
                .client
                .get_alert_message(scan_id, uri_id, &path.msg_id, true)
                .await?;

            match ctx.format {
                OutputFormat::Pretty | OutputFormat::Table => {
                    // Display banner
                    println!("{}\n", scan_context.format_banner());

                    let detail = AlertMessageDetail::new(message)
                        .with_context(&response.alert.name, &response.alert.severity);
                    println!("{}", detail.format_text());
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&message)?;
                    println!("{}", json);
                }
            }

            return Ok(());
        }
    }

    Err(crate::error::ApiError::NotFound(format!(
        "URI '{}' not found in scan. Use 'hawkop scan get {} --plugin-id <id>' to see available URIs.",
        uri_id, scan_id
    ))
    .into())
}

/// Format severity for display
fn format_severity(severity: &str) -> String {
    match severity.to_lowercase().as_str() {
        "high" => "High".to_string(),
        "medium" => "Medium".to_string(),
        "low" => "Low".to_string(),
        "informational" | "info" => "Info".to_string(),
        other => other.to_string(),
    }
}

/// Format triage status for display
fn format_triage_status(status: &str) -> String {
    match status {
        "UNKNOWN" => "New".to_string(),
        "PROMOTED" => "Triaged".to_string(),
        "ACCEPTED" | "RISK_ACCEPTED" => "Accepted".to_string(),
        "FALSE_POSITIVE" => "False Positive".to_string(),
        other => other.to_string(),
    }
}

/// Format scan status for display (normalize case, human-friendly)
fn format_scan_status(status: &str) -> String {
    match status.to_uppercase().as_str() {
        "STARTED" => "Running".to_string(),
        "COMPLETED" => "Complete".to_string(),
        "ERROR" => "Failed".to_string(),
        "UNKNOWN" => "Unknown".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::models::{AlertStats, AlertStatusStats, Scan};
    use std::collections::HashMap;

    // ========================================================================
    // Test Fixtures
    // ========================================================================

    /// Create a minimal ScanResult for testing
    fn make_scan(id: &str, app: &str, env: &str, status: &str) -> ScanResult {
        ScanResult {
            scan: Scan {
                id: id.to_string(),
                application_id: format!("app-{}", id),
                application_name: app.to_string(),
                env: env.to_string(),
                status: status.to_string(),
                timestamp: "1703721600000".to_string(), // 2023-12-28
                version: "5.0.0".to_string(),
                external_user_id: None,
            },
            scan_duration: Some("120".to_string()),
            url_count: Some(50),
            alert_stats: None,
            severity_stats: None,
            app_host: Some("https://example.com".to_string()),
            policy_name: None,
            tags: vec![],
            metadata: None,
        }
    }

    /// Create a ScanResult with alert stats for testing findings
    fn make_scan_with_findings(
        id: &str,
        high_new: u32,
        medium_new: u32,
        low_new: u32,
    ) -> ScanResult {
        let mut severity_stats = HashMap::new();
        if high_new > 0 {
            severity_stats.insert("High".to_string(), high_new);
        }
        if medium_new > 0 {
            severity_stats.insert("Medium".to_string(), medium_new);
        }
        if low_new > 0 {
            severity_stats.insert("Low".to_string(), low_new);
        }

        let mut scan = make_scan(id, "TestApp", "prod", "COMPLETED");
        scan.alert_stats = Some(AlertStats {
            total_alerts: high_new + medium_new + low_new,
            unique_alerts: high_new + medium_new + low_new,
            alert_status_stats: vec![AlertStatusStats {
                alert_status: "UNKNOWN".to_string(),
                total_count: high_new + medium_new + low_new,
                severity_stats,
            }],
        });
        scan
    }

    // ========================================================================
    // format_duration_seconds tests
    // ========================================================================

    #[test]
    fn test_format_duration_seconds_zero() {
        assert_eq!(format_duration_seconds("0"), "N/A");
    }

    #[test]
    fn test_format_duration_seconds_only_seconds() {
        assert_eq!(format_duration_seconds("45"), "45s");
    }

    #[test]
    fn test_format_duration_seconds_minutes_and_seconds() {
        assert_eq!(format_duration_seconds("125"), "2m 5s");
    }

    #[test]
    fn test_format_duration_seconds_hours() {
        assert_eq!(format_duration_seconds("3665"), "1h 1m 5s");
    }

    #[test]
    fn test_format_duration_seconds_invalid() {
        assert_eq!(format_duration_seconds("not-a-number"), "N/A");
    }

    // ========================================================================
    // format_scan_status tests
    // ========================================================================

    #[test]
    fn test_format_scan_status_started() {
        assert_eq!(format_scan_status("STARTED"), "Running");
        assert_eq!(format_scan_status("started"), "Running");
    }

    #[test]
    fn test_format_scan_status_completed() {
        assert_eq!(format_scan_status("COMPLETED"), "Complete");
        assert_eq!(format_scan_status("completed"), "Complete");
    }

    #[test]
    fn test_format_scan_status_error() {
        assert_eq!(format_scan_status("ERROR"), "Failed");
        assert_eq!(format_scan_status("error"), "Failed");
    }

    #[test]
    fn test_format_scan_status_unknown() {
        assert_eq!(format_scan_status("UNKNOWN"), "Unknown");
    }

    #[test]
    fn test_format_scan_status_passthrough() {
        assert_eq!(format_scan_status("CUSTOM_STATUS"), "CUSTOM_STATUS");
    }

    // ========================================================================
    // matches_status tests
    // ========================================================================

    #[test]
    fn test_matches_status_running() {
        let scan = make_scan("1", "App", "prod", "STARTED");
        assert!(matches_status(&scan, "running"));
        assert!(matches_status(&scan, "Running"));
        assert!(matches_status(&scan, "RUNNING"));
        assert!(!matches_status(&scan, "complete"));
    }

    #[test]
    fn test_matches_status_complete() {
        let scan = make_scan("1", "App", "prod", "COMPLETED");
        assert!(matches_status(&scan, "complete"));
        assert!(matches_status(&scan, "Complete"));
        assert!(!matches_status(&scan, "running"));
    }

    #[test]
    fn test_matches_status_failed() {
        let scan = make_scan("1", "App", "prod", "ERROR");
        assert!(matches_status(&scan, "failed"));
        assert!(matches_status(&scan, "Failed"));
        assert!(!matches_status(&scan, "complete"));
    }

    #[test]
    fn test_matches_status_partial_match() {
        let scan = make_scan("1", "App", "prod", "COMPLETED");
        // "comp" should match "complete"
        assert!(matches_status(&scan, "comp"));
    }

    // ========================================================================
    // get_new_findings tests
    // ========================================================================

    #[test]
    fn test_get_new_findings_no_stats() {
        let scan = make_scan("1", "App", "prod", "COMPLETED");
        assert_eq!(get_new_findings(&scan), (0, 0, 0));
    }

    #[test]
    fn test_get_new_findings_high_only() {
        let scan = make_scan_with_findings("1", 5, 0, 0);
        assert_eq!(get_new_findings(&scan), (5, 0, 0));
    }

    #[test]
    fn test_get_new_findings_all_severities() {
        let scan = make_scan_with_findings("1", 3, 5, 2);
        assert_eq!(get_new_findings(&scan), (3, 5, 2));
    }

    #[test]
    fn test_get_new_findings_empty_unknown() {
        let mut scan = make_scan("1", "App", "prod", "COMPLETED");
        // Create stats without UNKNOWN status
        scan.alert_stats = Some(AlertStats {
            total_alerts: 0u32,
            unique_alerts: 0u32,
            alert_status_stats: vec![AlertStatusStats {
                alert_status: "PROMOTED".to_string(),
                total_count: 5,
                severity_stats: HashMap::new(),
            }],
        });
        assert_eq!(get_new_findings(&scan), (0, 0, 0));
    }

    // ========================================================================
    // apply_status_filter tests
    // ========================================================================

    #[test]
    fn test_apply_status_filter_no_filter() {
        let scans = vec![
            make_scan("1", "App1", "prod", "COMPLETED"),
            make_scan("2", "App2", "prod", "STARTED"),
        ];
        let filters = ScanFilterArgs {
            app: vec![],
            env: vec![],
            status: None,
        };

        let result = apply_status_filter(scans.clone(), &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_status_filter_running() {
        let scans = vec![
            make_scan("1", "App1", "prod", "COMPLETED"),
            make_scan("2", "App2", "prod", "STARTED"),
            make_scan("3", "App3", "prod", "COMPLETED"),
        ];
        let filters = ScanFilterArgs {
            app: vec![],
            env: vec![],
            status: Some("running".to_string()),
        };

        let result = apply_status_filter(scans, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scan.id, "2");
    }

    #[test]
    fn test_apply_status_filter_complete() {
        let scans = vec![
            make_scan("1", "App1", "prod", "COMPLETED"),
            make_scan("2", "App2", "prod", "STARTED"),
            make_scan("3", "App3", "prod", "COMPLETED"),
        ];
        let filters = ScanFilterArgs {
            app: vec![],
            env: vec![],
            status: Some("complete".to_string()),
        };

        let result = apply_status_filter(scans, &filters);
        assert_eq!(result.len(), 2);
    }

    // ========================================================================
    // apply_sort tests
    // ========================================================================

    #[test]
    fn test_apply_sort_no_sort() {
        let scans = vec![
            make_scan("1", "Zebra", "prod", "COMPLETED"),
            make_scan("2", "Apple", "prod", "COMPLETED"),
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: None,
            sort_dir: None,
        };

        let result = apply_sort(scans.clone(), &pagination);
        // Should preserve original order
        assert_eq!(result[0].scan.id, "1");
        assert_eq!(result[1].scan.id, "2");
    }

    #[test]
    fn test_apply_sort_by_app_asc() {
        let scans = vec![
            make_scan("1", "Zebra", "prod", "COMPLETED"),
            make_scan("2", "Apple", "prod", "COMPLETED"),
            make_scan("3", "Mango", "prod", "COMPLETED"),
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("app".to_string()),
            sort_dir: Some(SortDir::Asc),
        };

        let result = apply_sort(scans, &pagination);
        assert_eq!(result[0].scan.application_name, "Apple");
        assert_eq!(result[1].scan.application_name, "Mango");
        assert_eq!(result[2].scan.application_name, "Zebra");
    }

    #[test]
    fn test_apply_sort_by_app_desc() {
        let scans = vec![
            make_scan("1", "Apple", "prod", "COMPLETED"),
            make_scan("2", "Zebra", "prod", "COMPLETED"),
            make_scan("3", "Mango", "prod", "COMPLETED"),
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("app".to_string()),
            sort_dir: Some(SortDir::Desc),
        };

        let result = apply_sort(scans, &pagination);
        assert_eq!(result[0].scan.application_name, "Zebra");
        assert_eq!(result[1].scan.application_name, "Mango");
        assert_eq!(result[2].scan.application_name, "Apple");
    }

    #[test]
    fn test_apply_sort_by_env() {
        let scans = vec![
            make_scan("1", "App", "prod", "COMPLETED"),
            make_scan("2", "App", "dev", "COMPLETED"),
            make_scan("3", "App", "staging", "COMPLETED"),
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("env".to_string()),
            sort_dir: Some(SortDir::Asc),
        };

        let result = apply_sort(scans, &pagination);
        assert_eq!(result[0].scan.env, "dev");
        assert_eq!(result[1].scan.env, "prod");
        assert_eq!(result[2].scan.env, "staging");
    }

    #[test]
    fn test_apply_sort_by_status() {
        let scans = vec![
            make_scan("1", "App", "prod", "STARTED"),
            make_scan("2", "App", "prod", "COMPLETED"),
            make_scan("3", "App", "prod", "ERROR"),
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("status".to_string()),
            sort_dir: Some(SortDir::Asc),
        };

        let result = apply_sort(scans, &pagination);
        assert_eq!(result[0].scan.status, "COMPLETED");
        assert_eq!(result[1].scan.status, "ERROR");
        assert_eq!(result[2].scan.status, "STARTED");
    }

    #[test]
    fn test_apply_sort_by_findings() {
        let scans = vec![
            make_scan_with_findings("1", 1, 0, 0),  // 1 high
            make_scan_with_findings("2", 5, 0, 0),  // 5 high
            make_scan_with_findings("3", 0, 10, 0), // 0 high, 10 medium
        ];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("findings".to_string()),
            sort_dir: Some(SortDir::Desc),
        };

        let result = apply_sort(scans, &pagination);
        // 5 high should be first (highest priority)
        assert_eq!(result[0].scan.id, "2");
        // 1 high should be second
        assert_eq!(result[1].scan.id, "1");
        // 0 high, 10 medium should be last
        assert_eq!(result[2].scan.id, "3");
    }

    #[test]
    fn test_apply_sort_by_duration() {
        let mut scan1 = make_scan("1", "App", "prod", "COMPLETED");
        scan1.scan_duration = Some("300".to_string()); // 5 min
        let mut scan2 = make_scan("2", "App", "prod", "COMPLETED");
        scan2.scan_duration = Some("60".to_string()); // 1 min
        let mut scan3 = make_scan("3", "App", "prod", "COMPLETED");
        scan3.scan_duration = Some("600".to_string()); // 10 min

        let scans = vec![scan1, scan2, scan3];
        let pagination = PaginationArgs {
            limit: None,
            page: None,
            sort_by: Some("duration".to_string()),
            sort_dir: Some(SortDir::Asc),
        };

        let result = apply_sort(scans, &pagination);
        assert_eq!(result[0].scan.id, "2"); // 1 min
        assert_eq!(result[1].scan.id, "1"); // 5 min
        assert_eq!(result[2].scan.id, "3"); // 10 min
    }

    // ========================================================================
    // ScanContext tests
    // ========================================================================

    #[test]
    fn test_scan_context_from_scan_result() {
        let scan = make_scan("scan-123", "MyApp", "production", "COMPLETED");
        let context = ScanContext::from_scan_result(&scan);

        assert_eq!(context.app_name, "MyApp");
        assert_eq!(context.env, "production");
        assert_eq!(context.scan_id, "scan-123");
        assert_eq!(context.status, "COMPLETED");
        assert!(context.host.is_some());
    }
}
