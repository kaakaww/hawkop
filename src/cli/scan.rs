//! Scan management commands

use crate::cli::{CommandContext, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
use crate::client::{PaginationParams, ScanFilterParams, ScanResult, StackHawkApi};
use crate::error::Result;
use crate::models::ScanDisplay;
use crate::output::Formattable;

/// Default limit for scan list display
const DEFAULT_SCAN_LIMIT: usize = 25;

/// API's actual max page size for scans endpoint
const SCAN_API_PAGE_SIZE: usize = 100;

/// Max scans to fetch when sorting (to avoid runaway queries)
const MAX_SORT_FETCH: usize = 10_000;

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

    // Fetch scans, paginating if needed (API max is 100 per page)
    // Note: Don't pass sort params to API - it has limited field support
    // We sort client-side instead for better UX
    let mut all_scans: Vec<ScanResult> = Vec::new();
    let mut page = pagination.page.unwrap_or(0);

    loop {
        let pagination_params = PaginationParams::new()
            .page_size(SCAN_API_PAGE_SIZE)
            .page(page);

        let scans = ctx
            .client
            .list_scans(org_id, Some(&pagination_params), filter_params.as_ref())
            .await?;

        let batch_size = scans.len();
        all_scans.extend(scans);

        // When status filtering, count filtered results to know when we have enough
        // (status filter ratio is unknown, so we can't predict from total count)
        let effective_count = if has_status_filter {
            count_status_matches(&all_scans, filters)
        } else {
            all_scans.len()
        };

        // Stop if we have enough filtered results or no more API results
        if effective_count >= target_count || batch_size < SCAN_API_PAGE_SIZE {
            break;
        }

        page += 1;
    }

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

/// Count how many scans match the status filter.
/// Used during pagination to know when we have enough filtered results.
fn count_status_matches(scans: &[ScanResult], filters: &ScanFilterArgs) -> usize {
    let Some(ref status_filter) = filters.status else {
        return scans.len();
    };
    scans
        .iter()
        .filter(|scan| matches_status(scan, status_filter))
        .count()
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
                let a_dur = a.scan_duration.as_deref().unwrap_or("0");
                let b_dur = b.scan_duration.as_deref().unwrap_or("0");
                a_dur.cmp(b_dur)
            }
            "id" => a.scan.id.cmp(&b.scan.id),
            _ => std::cmp::Ordering::Equal,
        };

        if descending { cmp.reverse() } else { cmp }
    });

    scans
}
