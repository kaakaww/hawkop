//! Application management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::{Application, PaginationParams, StackHawkApi, fetch_remaining_pages};
use crate::error::Result;
use crate::models::AppDisplay;
use crate::output::Formattable;

/// Page size for apps endpoint
const APP_API_PAGE_SIZE: usize = 100;

/// Max concurrent requests for parallel fetching
const PARALLEL_FETCH_LIMIT: usize = 32;

/// Run the app list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    app_type: Option<&str>,
    pagination: &PaginationArgs,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
    let org_id = ctx.require_org_id()?;

    // Fetch apps using totalCount-based parallel pagination
    let start_page = pagination.page.unwrap_or(0);
    let first_params = PaginationParams::new()
        .page_size(APP_API_PAGE_SIZE)
        .page(start_page);

    debug!(
        "Fetching first page of apps (page={}, pageSize={})",
        start_page, APP_API_PAGE_SIZE
    );

    let first_response = ctx
        .client
        .list_apps_paged(org_id, Some(&first_params))
        .await?;

    let mut all_apps = first_response.items;
    debug!(
        "First page returned {} apps, totalCount={:?}",
        all_apps.len(),
        first_response.total_count
    );

    // Fetch remaining pages if totalCount indicates more
    if let Some(total_count) = first_response.total_count {
        let total_pages = total_count.div_ceil(APP_API_PAGE_SIZE);

        if total_pages > 1 {
            let remaining_pages: Vec<usize> = (start_page + 1..start_page + total_pages).collect();

            if !remaining_pages.is_empty() {
                debug!(
                    "Fetching {} remaining pages in parallel",
                    remaining_pages.len()
                );

                let client = ctx.client.clone();
                let org = org_id.to_string();

                let remaining_apps = fetch_remaining_pages(
                    remaining_pages,
                    move |page| {
                        let c = client.clone();
                        let o = org.clone();
                        async move {
                            let params = PaginationParams::new()
                                .page_size(APP_API_PAGE_SIZE)
                                .page(page);
                            c.list_apps(&o, Some(&params)).await
                        }
                    },
                    PARALLEL_FETCH_LIMIT,
                )
                .await?;

                all_apps.extend(remaining_apps);
            }
        }
    }

    debug!("Total apps fetched: {}", all_apps.len());

    // Apply type filter if specified
    let filtered_apps = filter_by_type(all_apps, app_type);
    debug!("Apps after type filter: {}", filtered_apps.len());

    // Apply limit if specified
    let limited_apps = if let Some(limit) = pagination.limit {
        filtered_apps.into_iter().take(limit).collect()
    } else {
        filtered_apps
    };

    let display_apps: Vec<AppDisplay> = limited_apps.into_iter().map(AppDisplay::from).collect();
    display_apps.print(ctx.format)?;

    Ok(())
}

/// Filter applications by type (cloud or standard)
fn filter_by_type(apps: Vec<Application>, app_type: Option<&str>) -> Vec<Application> {
    match app_type {
        Some(filter) => {
            let filter_upper = filter.to_uppercase();
            apps.into_iter()
                .filter(|app| {
                    let app_type = app
                        .application_type
                        .as_ref()
                        .map(|t| t.to_uppercase())
                        .unwrap_or_else(|| "STANDARD".to_string());
                    app_type == filter_upper
                })
                .collect()
        }
        None => apps,
    }
}
