//! Application management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::models::Application;
use crate::client::{ListingApi, PaginationParams, fetch_remaining_pages};
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
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Test Fixtures
    // ========================================================================

    /// Create a minimal Application for testing
    fn make_app(id: &str, name: &str, app_type: Option<&str>) -> Application {
        Application {
            id: id.to_string(),
            name: name.to_string(),
            env: Some("prod".to_string()),
            risk_level: None,
            status: Some("ACTIVE".to_string()),
            organization_id: Some("org-123".to_string()),
            application_type: app_type.map(|t| t.to_string()),
            cloud_scan_target: None,
        }
    }

    // ========================================================================
    // filter_by_type tests
    // ========================================================================

    #[test]
    fn test_filter_by_type_no_filter() {
        let apps = vec![
            make_app("1", "App One", Some("STANDARD")),
            make_app("2", "App Two", Some("CLOUD")),
            make_app("3", "App Three", None),
        ];

        let result = filter_by_type(apps, None);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_filter_by_type_standard() {
        let apps = vec![
            make_app("1", "Standard App", Some("STANDARD")),
            make_app("2", "Cloud App", Some("CLOUD")),
            make_app("3", "Untyped App", None), // defaults to STANDARD
        ];

        let result = filter_by_type(apps, Some("standard"));
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|a| a.id == "1"));
        assert!(result.iter().any(|a| a.id == "3")); // untyped defaults to standard
    }

    #[test]
    fn test_filter_by_type_cloud() {
        let apps = vec![
            make_app("1", "Standard App", Some("STANDARD")),
            make_app("2", "Cloud App", Some("CLOUD")),
            make_app("3", "Another Cloud", Some("CLOUD")),
        ];

        let result = filter_by_type(apps, Some("cloud"));
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|a| a.application_type.as_deref() == Some("CLOUD"))
        );
    }

    #[test]
    fn test_filter_by_type_case_insensitive() {
        let apps = vec![
            make_app("1", "Cloud App", Some("CLOUD")),
            make_app("2", "Standard App", Some("STANDARD")),
        ];

        // All these variations should match the cloud app
        assert_eq!(filter_by_type(apps.clone(), Some("CLOUD")).len(), 1);
        assert_eq!(filter_by_type(apps.clone(), Some("cloud")).len(), 1);
        assert_eq!(filter_by_type(apps.clone(), Some("Cloud")).len(), 1);
    }

    #[test]
    fn test_filter_by_type_empty_list() {
        let apps: Vec<Application> = vec![];
        let result = filter_by_type(apps, Some("standard"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_by_type_no_matches() {
        let apps = vec![
            make_app("1", "Standard App", Some("STANDARD")),
            make_app("2", "Another Standard", Some("STANDARD")),
        ];

        let result = filter_by_type(apps, Some("cloud"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_by_type_preserves_order() {
        let apps = vec![
            make_app("3", "Third", Some("STANDARD")),
            make_app("1", "First", Some("STANDARD")),
            make_app("2", "Second", Some("STANDARD")),
        ];

        let result = filter_by_type(apps, Some("standard"));
        assert_eq!(result[0].id, "3");
        assert_eq!(result[1].id, "1");
        assert_eq!(result[2].id, "2");
    }

    #[test]
    fn test_filter_by_type_untyped_defaults_to_standard() {
        let apps = vec![make_app("1", "No Type", None)];

        // Should match standard filter
        let result = filter_by_type(apps.clone(), Some("standard"));
        assert_eq!(result.len(), 1);

        // Should NOT match cloud filter
        let result = filter_by_type(apps, Some("cloud"));
        assert!(result.is_empty());
    }
}
