//! Parallel fetching utilities for paginated API requests.
//!
//! Provides helpers to fetch multiple pages in parallel using totalCount
//! from the first response.

use std::future::Future;
use std::pin::Pin;

use futures::stream::{FuturesUnordered, StreamExt};
use log::debug;

use crate::error::Result;

/// Type alias for boxed futures used in parallel fetching
type PageFuture<T> = Pin<Box<dyn Future<Output = (usize, Result<Vec<T>>)> + Send>>;

/// Fetch all remaining pages in parallel after the first page.
///
/// Uses the `remaining_pages` from the first response to spawn parallel requests
/// for all remaining pages, up to `max_concurrent` at a time.
///
/// # Arguments
///
/// * `remaining_pages` - Page numbers to fetch (from `PagedResponse::remaining_pages()`)
/// * `fetch_page` - Async function that fetches a single page by number
/// * `max_concurrent` - Maximum number of concurrent requests
///
/// # Returns
///
/// A vector containing all items from all remaining pages, in arrival order.
///
/// # Example
///
/// ```ignore
/// let first_page = client.list_scans_paged(&org_id, Some(&params), None).await?;
/// let mut all_scans = first_page.items;
///
/// if first_page.has_more_pages() {
///     let remaining = fetch_remaining_pages(
///         first_page.remaining_pages(),
///         |page| {
///             let c = client.clone();
///             let o = org_id.clone();
///             async move {
///                 let params = PaginationParams::new().page_size(100).page(page);
///                 c.list_scans(&o, Some(&params), None).await
///             }
///         },
///         32,
///     ).await?;
///     all_scans.extend(remaining);
/// }
/// ```
pub async fn fetch_remaining_pages<T, F, Fut>(
    remaining_pages: Vec<usize>,
    fetch_page: F,
    max_concurrent: usize,
) -> Result<Vec<T>>
where
    T: Send + 'static,
    F: Fn(usize) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>>> + Send + 'static,
{
    if remaining_pages.is_empty() {
        return Ok(Vec::new());
    }

    debug!(
        "Fetching {} remaining pages with max {} concurrent",
        remaining_pages.len(),
        max_concurrent
    );

    let mut all_items = Vec::new();
    let mut futures: FuturesUnordered<PageFuture<T>> = FuturesUnordered::new();
    let mut pending_pages = remaining_pages.into_iter();

    // Helper to create a boxed future
    let make_future = |page: usize, f: &F| -> PageFuture<T> {
        let fut = f(page);
        Box::pin(async move {
            let result = fut.await;
            (page, result)
        })
    };

    // Seed initial batch up to max_concurrent
    for page in pending_pages.by_ref().take(max_concurrent) {
        debug!("Spawning initial request for page {}", page);
        futures.push(make_future(page, &fetch_page));
    }

    // Process results and spawn new requests to maintain concurrency
    while let Some((page, result)) = futures.next().await {
        let items = result?;
        debug!("Page {} returned {} items", page, items.len());
        all_items.extend(items);

        // Spawn next page request if any remaining
        if let Some(next_page) = pending_pages.next() {
            debug!("Spawning request for page {}", next_page);
            futures.push(make_future(next_page, &fetch_page));
        }
    }

    debug!("Fetched {} total items from remaining pages", all_items.len());
    Ok(all_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_fetch_remaining_pages_empty() {
        let result: Result<Vec<String>> =
            fetch_remaining_pages(vec![], |_page| async { Ok(vec![]) }, 10).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_fetch_remaining_pages_single() {
        let result: Result<Vec<String>> = fetch_remaining_pages(
            vec![1],
            |page| async move { Ok(vec![format!("item-{}", page)]) },
            10,
        )
        .await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "item-1");
    }

    #[tokio::test]
    async fn test_fetch_remaining_pages_multiple() {
        let result: Result<Vec<String>> = fetch_remaining_pages(
            vec![1, 2, 3],
            |page| async move { Ok(vec![format!("item-{}-a", page), format!("item-{}-b", page)]) },
            10,
        )
        .await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 6); // 3 pages * 2 items each
    }

    #[tokio::test]
    async fn test_fetch_remaining_pages_respects_concurrency() {
        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));

        let cc = concurrent_count.clone();
        let mo = max_observed.clone();

        let result: Result<Vec<usize>> = fetch_remaining_pages(
            vec![1, 2, 3, 4, 5],
            move |page| {
                let cc = cc.clone();
                let mo = mo.clone();
                async move {
                    // Track concurrent requests
                    let current = cc.fetch_add(1, Ordering::SeqCst) + 1;
                    mo.fetch_max(current, Ordering::SeqCst);

                    // Simulate some work
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                    cc.fetch_sub(1, Ordering::SeqCst);
                    Ok(vec![page])
                }
            },
            2, // Only 2 concurrent
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
        // Max concurrent should not exceed our limit
        assert!(max_observed.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    async fn test_fetch_remaining_pages_propagates_errors() {
        let result: Result<Vec<String>> = fetch_remaining_pages(
            vec![1, 2, 3],
            |page| async move {
                if page == 2 {
                    Err(crate::error::ApiError::ServerError("test error".to_string()).into())
                } else {
                    Ok(vec![format!("item-{}", page)])
                }
            },
            10,
        )
        .await;

        assert!(result.is_err());
    }
}
