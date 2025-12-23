//! Pagination helpers for API requests
//!
//! Provides types and utilities for handling paginated API responses.
//!
//! These types are infrastructure for future paginated commands (scan list, finding list, etc.)
//! and will be used once those commands are implemented.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Maximum page size supported by StackHawk API.
/// Using this as default minimizes API calls.
pub const MAX_PAGE_SIZE: usize = 1000;

/// Pagination parameters for API requests.
///
/// Use the builder pattern to configure pagination options.
///
/// # Example
/// ```ignore
/// let params = PaginationParams::new()
///     .page_size(100)
///     .page(2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    /// Number of items per page (default: 1000, max: 1000)
    pub page_size: Option<usize>,
    /// Page number (0-indexed for some endpoints, 1-indexed for others)
    pub page: Option<usize>,
    /// Sort field name
    pub sort_by: Option<String>,
    /// Sort order
    pub sort_order: Option<SortOrder>,
}

/// Sort order for paginated requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order (A-Z, 0-9, oldest first)
    Asc,
    /// Descending order (Z-A, 9-0, newest first)
    Desc,
}

impl PaginationParams {
    /// Create new pagination params with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page size (items per page).
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Set the page number.
    pub fn page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the sort field.
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    /// Set the sort order.
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Convert to query string parameters.
    ///
    /// Returns a vector of (key, value) pairs suitable for URL encoding.
    /// Uses StackHawk API parameter names:
    /// - `pageSize`: number of elements per page (defaults to MAX_PAGE_SIZE to minimize API calls)
    /// - `pageToken`: page number to start at (0-indexed)
    /// - `sortField`: field to sort by
    /// - `sortDir`: 'asc' or 'desc'
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();

        // Always include pageSize, defaulting to max to minimize API calls
        let size = self.page_size.unwrap_or(MAX_PAGE_SIZE);
        params.push(("pageSize", size.to_string()));

        if let Some(page) = self.page {
            params.push(("pageToken", page.to_string()));
        }

        if let Some(ref field) = self.sort_by {
            params.push(("sortField", field.clone()));
        }

        if let Some(order) = self.sort_order {
            let order_str = match order {
                SortOrder::Asc => "asc",
                SortOrder::Desc => "desc",
            };
            params.push(("sortDir", order_str.to_string()));
        }

        params
    }

    /// Check if any pagination parameters are set.
    pub fn is_empty(&self) -> bool {
        self.page_size.is_none()
            && self.page.is_none()
            && self.sort_by.is_none()
            && self.sort_order.is_none()
    }
}

/// Response metadata for paginated results.
///
/// Included in API responses that support pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    /// Total number of items across all pages
    #[serde(default)]
    pub total_count: Option<usize>,

    /// Total number of pages
    #[serde(default)]
    pub page_count: Option<usize>,

    /// Current page number
    #[serde(default)]
    pub current_page: Option<usize>,

    /// Items per page
    #[serde(default)]
    pub page_size: Option<usize>,

    /// Whether there are more pages
    #[serde(default)]
    pub has_more: Option<bool>,
}

impl PaginationMeta {
    /// Check if there are more pages to fetch.
    pub fn has_next_page(&self) -> bool {
        if let Some(has_more) = self.has_more {
            return has_more;
        }

        if let (Some(current), Some(total)) = (self.current_page, self.page_count) {
            return current < total;
        }

        false
    }
}

/// A paginated response containing data and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    pub data: Vec<T>,

    /// Pagination metadata
    #[serde(default)]
    pub pagination: Option<PaginationMeta>,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response.
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            pagination: None,
        }
    }

    /// Create a paginated response with metadata.
    pub fn with_pagination(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self {
            data,
            pagination: Some(pagination),
        }
    }

    /// Check if there are more pages to fetch.
    pub fn has_next_page(&self) -> bool {
        self.pagination
            .as_ref()
            .map(|p| p.has_next_page())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::new();
        assert!(params.is_empty());

        // Even with no params set, to_query_params includes default pageSize
        let query = params.to_query_params();
        assert_eq!(query.len(), 1);
        assert!(query.contains(&("pageSize", MAX_PAGE_SIZE.to_string())));
    }

    #[test]
    fn test_pagination_params_builder() {
        let params = PaginationParams::new()
            .page_size(100)
            .page(2)
            .sort_by("name")
            .sort_order(SortOrder::Desc);

        assert!(!params.is_empty());
        assert_eq!(params.page_size, Some(100));
        assert_eq!(params.page, Some(2));
        assert_eq!(params.sort_by, Some("name".to_string()));
        assert_eq!(params.sort_order, Some(SortOrder::Desc));
    }

    #[test]
    fn test_pagination_params_to_query() {
        let params = PaginationParams::new().page_size(50).page(1);

        let query = params.to_query_params();
        assert_eq!(query.len(), 2);
        assert!(query.contains(&("pageSize", "50".to_string())));
        assert!(query.contains(&("pageToken", "1".to_string())));
    }

    #[test]
    fn test_pagination_meta_has_next_page() {
        // Using has_more flag
        let meta = PaginationMeta {
            total_count: None,
            page_count: None,
            current_page: None,
            page_size: None,
            has_more: Some(true),
        };
        assert!(meta.has_next_page());

        // Using page counts
        let meta = PaginationMeta {
            total_count: Some(100),
            page_count: Some(10),
            current_page: Some(5),
            page_size: Some(10),
            has_more: None,
        };
        assert!(meta.has_next_page());

        // Last page
        let meta = PaginationMeta {
            total_count: Some(100),
            page_count: Some(10),
            current_page: Some(10),
            page_size: Some(10),
            has_more: None,
        };
        assert!(!meta.has_next_page());
    }

    #[test]
    fn test_paginated_response() {
        let response: PaginatedResponse<String> =
            PaginatedResponse::new(vec!["a".to_string(), "b".to_string()]);

        assert_eq!(response.data.len(), 2);
        assert!(!response.has_next_page());

        let meta = PaginationMeta {
            total_count: Some(100),
            page_count: Some(10),
            current_page: Some(1),
            page_size: Some(10),
            has_more: Some(true),
        };

        let response = PaginatedResponse::with_pagination(vec!["a".to_string()], meta);
        assert!(response.has_next_page());
    }
}
