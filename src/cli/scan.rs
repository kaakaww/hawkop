//! Scan management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
use crate::client::{
    PaginationParams, ScanFilterParams, ScanResult, StackHawkApi, fetch_remaining_pages,
};
use crate::error::Result;
use crate::models::ScanDisplay;
use crate::output::Formattable;

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
        .list_scans_paged(&org_id, Some(&first_params), filter_params.as_ref())
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
                .list_scans(&org_id, Some(&params), filter_params.as_ref())
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
