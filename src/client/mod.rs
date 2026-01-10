//! StackHawk API client
//!
//! This module provides the API client for interacting with StackHawk services.
//! The API surface is organized into focused sub-traits:
//!
//! - [`AuthApi`] - Authentication operations
//! - [`ListingApi`] - Collection listing operations
//! - [`ScanDetailApi`] - Scan drill-down operations
//!
//! The [`StackHawkApi`] super-trait combines all three for convenience.

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

/// StackHawk API client super-trait
///
/// This trait combines all API capabilities:
/// - [`AuthApi`] for authentication
/// - [`ListingApi`] for listing resources
/// - [`ScanDetailApi`] for scan drill-down
///
/// Any type implementing all three sub-traits automatically implements this trait.
pub trait StackHawkApi: AuthApi + ListingApi + ScanDetailApi {}

// Blanket implementation: any type implementing all sub-traits gets StackHawkApi
impl<T: AuthApi + ListingApi + ScanDetailApi> StackHawkApi for T {}
