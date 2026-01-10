//! StackHawk API client
//!
//! This module provides the API client for interacting with StackHawk services.
//! The API surface is organized into focused sub-traits:
//!
//! - [`AuthApi`] - Authentication operations
//! - [`ListingApi`] - Collection listing operations
//! - [`ScanDetailApi`] - Scan drill-down operations

pub mod api;
#[cfg(test)]
pub mod mock;
pub mod models;
pub mod pagination;
pub mod parallel;
pub mod rate_limit;
pub mod stackhawk;

// Re-export sub-traits
pub use api::{AuthApi, ListingApi, ScanDetailApi};

#[cfg(test)]
#[allow(unused_imports)]
pub use mock::MockStackHawkClient;
#[allow(unused_imports)]
pub use pagination::{
    MAX_PAGE_SIZE, PagedResponse, PaginatedResponse, PaginationMeta, PaginationParams,
    ScanFilterParams, SortOrder,
};
#[allow(unused_imports)]
pub use parallel::fetch_remaining_pages;
pub use stackhawk::StackHawkClient;
