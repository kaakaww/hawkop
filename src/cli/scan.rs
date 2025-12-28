//! Scan management commands

use chrono::{TimeZone, Utc};
use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
use crate::client::{
    PaginationParams, ScanFilterParams, ScanResult, StackHawkApi, fetch_remaining_pages,
};
use crate::error::Result;
use crate::models::{
    AlertDetail, AlertDisplay, AlertFindingDisplay, AlertMessageDetail, ScanDisplay, ScanOverview,
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
    /// HawkScan version
    pub version: String,
    /// Scan timestamp (Unix epoch milliseconds)
    pub timestamp: String,
    /// Scan duration in seconds
    pub duration: Option<String>,
}

impl ScanContext {
    /// Create scan context from a ScanResult
    pub fn from_scan_result(scan: &ScanResult) -> Self {
        Self {
            app_name: scan.scan.application_name.clone(),
            env: scan.scan.env.clone(),
            host: scan.app_host.clone(),
            version: scan.scan.version.clone(),
            timestamp: scan.scan.timestamp.clone(),
            duration: scan.scan_duration.clone(),
        }
    }

    /// Format the scan banner for display
    pub fn format_banner(&self) -> String {
        let mut lines = Vec::new();

        // Header line: App / Environment
        let header = format!("── {} / {} ", self.app_name, self.env);
        let padding = "─".repeat(72_usize.saturating_sub(header.len()));
        lines.push(format!("{}{}", header, padding));

        // Line 2: Host and HawkScan version
        let host_str = self.host.as_deref().unwrap_or("N/A");
        let version_str = if self.version.is_empty() { "N/A" } else { &self.version };
        lines.push(format!(" Host: {}  HawkScan: {}", host_str, version_str));

        // Line 3: Date and Duration
        let date_str = format_timestamp_local(&self.timestamp);
        let duration_str = self.duration.as_ref()
            .map(|d| format_duration_seconds(d))
            .unwrap_or_else(|| "N/A".to_string());
        lines.push(format!(" Date: {}  Duration: {}", date_str, duration_str));

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
        8 => "CST",  // China Standard Time
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
const DEFAULT_SCAN_LIMIT: usize = 25;

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
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
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
// Scan View / Drill-Down Commands
// ============================================================================

/// Drill-down depth parsed from positional arguments
#[derive(Debug)]
enum DrillDown {
    /// Show scan overview (no args)
    ScanOverview,
    /// List all alerts (args: ["alerts"])
    AlertsList,
    /// Show specific alert with paths (args: ["alert", <plugin_id>])
    AlertDetail { plugin_id: String },
    /// Show specific path detail (args: ["alert", <plugin_id>, "uri", <uri_id>])
    UriDetail { plugin_id: String, uri_id: String },
    /// Show HTTP message (args: ["alert", <plugin_id>, "uri", <uri_id>, "message"])
    Message { plugin_id: String, uri_id: String },
}

impl DrillDown {
    /// Parse drill-down command from positional arguments
    fn parse(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Ok(DrillDown::ScanOverview);
        }

        let first = args[0].to_lowercase();

        match first.as_str() {
            "alerts" => Ok(DrillDown::AlertsList),
            "alert" => {
                if args.len() < 2 {
                    return Err(crate::error::ApiError::BadRequest(
                        "Usage: scan view <id> alert <plugin-id> [uri <uri-id> [message]]".to_string(),
                    )
                    .into());
                }

                let plugin_id = args[1].clone();

                if args.len() == 2 {
                    // scan view <id> alert <plugin>
                    return Ok(DrillDown::AlertDetail { plugin_id });
                }

                // Check for "uri" keyword
                let third = args[2].to_lowercase();
                if third != "uri" {
                    return Err(crate::error::ApiError::BadRequest(
                        format!("Expected 'uri' keyword, got '{}'. Usage: alert <plugin> uri <uri-id> [message]", args[2])
                    )
                    .into());
                }

                if args.len() < 4 {
                    return Err(crate::error::ApiError::BadRequest(
                        "Usage: alert <plugin-id> uri <uri-id> [message]".to_string(),
                    )
                    .into());
                }

                let uri_id = args[3].clone();

                if args.len() == 4 {
                    // scan view <id> alert <plugin> uri <uri>
                    Ok(DrillDown::UriDetail { plugin_id, uri_id })
                } else {
                    // scan view <id> alert <plugin> uri <uri> message
                    let fifth = args[4].to_lowercase();
                    if fifth == "message" || fifth == "msg" {
                        Ok(DrillDown::Message { plugin_id, uri_id })
                    } else {
                        Ok(DrillDown::UriDetail { plugin_id, uri_id })
                    }
                }
            }
            _ => Err(crate::error::ApiError::BadRequest(format!(
                "Unknown drill-down command: '{}'. Use 'alerts' or 'alert <plugin-id>'",
                first
            ))
            .into()),
        }
    }
}

/// Run the scan view/drill-down command
///
/// Supports positional cascade for navigating scan results:
/// - `scan <id>` - Scan overview
/// - `scan <id> alerts` - List all alerts
/// - `scan <id> alert <plugin>` - Alert detail with paths
/// - `scan <id> alert <plugin> <uri> message` - HTTP request/response
pub async fn view(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    scan_id: &str,
    args: &[String],
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
    let org_id = ctx.require_org_id()?;

    let drill_down = DrillDown::parse(args)?;
    debug!("Drill-down: {:?}", drill_down);

    match drill_down {
        DrillDown::ScanOverview => show_scan_overview(&ctx, org_id, scan_id).await,
        DrillDown::AlertsList => show_alerts_list(&ctx, org_id, scan_id).await,
        DrillDown::AlertDetail { plugin_id } => {
            show_alert_detail(&ctx, org_id, scan_id, &plugin_id).await
        }
        DrillDown::UriDetail { plugin_id, uri_id } => {
            show_uri_detail(&ctx, org_id, scan_id, &plugin_id, &uri_id).await
        }
        DrillDown::Message { plugin_id, uri_id } => {
            show_message(&ctx, org_id, scan_id, &plugin_id, &uri_id).await
        }
    }
}

/// Show scan overview (scan <id>)
async fn show_scan_overview(ctx: &CommandContext, org_id: &str, scan_id: &str) -> Result<()> {
    debug!("Fetching scan overview for {}", scan_id);
    let scan = ctx.client.get_scan(org_id, scan_id).await?;

    match ctx.format {
        OutputFormat::Table => {
            // Display banner
            let scan_context = ScanContext::from_scan_result(&scan);
            println!("{}\n", scan_context.format_banner());

            let overview = ScanOverview::new(scan);
            println!("{}", overview.format_text(scan_id));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&scan)?;
            println!("{}", json);
        }
    }

    Ok(())
}

/// Show alerts list (scan <id> alerts)
async fn show_alerts_list(ctx: &CommandContext, org_id: &str, scan_id: &str) -> Result<()> {
    debug!("Fetching alerts for scan {}", scan_id);

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    let alerts = ctx.client.list_scan_alerts(scan_id, None).await?;
    let display_alerts: Vec<AlertDisplay> = alerts.into_iter().map(AlertDisplay::from).collect();

    match ctx.format {
        OutputFormat::Table => {
            // Display banner
            println!("{}\n", scan_context.format_banner());

            display_alerts.print(ctx.format)?;

            // Navigation hint
            if !display_alerts.is_empty() {
                println!("\n→ hawkop scan view {} alert <plugin-id>", scan_id);
            }
        }
        OutputFormat::Json => {
            display_alerts.print(ctx.format)?;
        }
    }

    Ok(())
}

/// Show alert detail with paths (scan <id> alert <plugin>)
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
        OutputFormat::Table => {
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

            display_paths.print(ctx.format)?;

            // Navigation hint
            if !display_paths.is_empty() {
                println!(
                    "\n→ hawkop scan view {} alert {} uri <uri-id>",
                    scan_id, plugin_id
                );
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&response)?;
            println!("{}", json);
        }
    }

    Ok(())
}

/// Show URI detail (scan <id> alert <plugin> uri <uri-id>)
async fn show_uri_detail(
    ctx: &CommandContext,
    org_id: &str,
    scan_id: &str,
    plugin_id: &str,
    uri_id: &str,
) -> Result<()> {
    debug!(
        "Fetching URI detail for scan {} alert {} uri {}",
        scan_id, plugin_id, uri_id
    );

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    // Get alert to find the specific path
    let response = ctx
        .client
        .get_alert_with_paths(scan_id, plugin_id, None)
        .await?;

    let path = response
        .application_scan_alert_uris
        .iter()
        .find(|p| p.alert_uri_id == uri_id)
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("URI not found: {}", uri_id)))?;

    // Fetch message to get evidence and other_info
    let message = ctx
        .client
        .get_alert_message(scan_id, uri_id, &path.msg_id, false)
        .await?;

    match ctx.format {
        OutputFormat::Table => {
            // Display banner
            println!("{}\n", scan_context.format_banner());

            // Alert context
            println!(
                "{} [{}]",
                response.alert.name,
                format_severity(&response.alert.severity)
            );
            println!("────────────────────────────────────────────────────────────────────────");

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
                // Wrap long other_info text
                for line in other_info.lines() {
                    println!("  {}", line);
                }
            }

            println!(
                "\n→ hawkop scan view {} alert {} uri {} message",
                scan_id, plugin_id, uri_id
            );
        }
        OutputFormat::Json => {
            // Include both path and message data for JSON
            let combined = serde_json::json!({
                "uri": path,
                "evidence": message.evidence,
                "other_info": message.other_info,
            });
            let json = serde_json::to_string_pretty(&combined)?;
            println!("{}", json);
        }
    }

    Ok(())
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

/// Show HTTP message (scan <id> alert <plugin> uri <uri-id> message)
async fn show_message(
    ctx: &CommandContext,
    org_id: &str,
    scan_id: &str,
    plugin_id: &str,
    uri_id: &str,
) -> Result<()> {
    debug!(
        "Fetching message for scan {} alert {} uri {}",
        scan_id, plugin_id, uri_id
    );

    // Fetch scan for banner context
    let scan = ctx.client.get_scan(org_id, scan_id).await?;
    let scan_context = ScanContext::from_scan_result(&scan);

    // Get the alert to find the message ID and context
    let alert_response = ctx
        .client
        .get_alert_with_paths(scan_id, plugin_id, None)
        .await?;

    let path = alert_response
        .application_scan_alert_uris
        .iter()
        .find(|p| p.alert_uri_id == uri_id)
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("URI not found: {}", uri_id)))?;

    // Fetch the message with curl validation command
    let message = ctx
        .client
        .get_alert_message(scan_id, uri_id, &path.msg_id, true)
        .await?;

    match ctx.format {
        OutputFormat::Table => {
            // Display banner
            println!("{}\n", scan_context.format_banner());

            let detail = AlertMessageDetail::new(message)
                .with_context(&alert_response.alert.name, &alert_response.alert.severity);
            println!("{}", detail.format_text());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&message)?;
            println!("{}", json);
        }
    }

    Ok(())
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
