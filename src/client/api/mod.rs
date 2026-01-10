//! API trait definitions split by responsibility
//!
//! This module organizes the StackHawk API surface into focused sub-traits:
//! - [`AuthApi`] - Authentication operations
//! - [`ListingApi`] - Collection listing operations
//! - [`ScanDetailApi`] - Scan drill-down operations
//!
//! The [`StackHawkApi`](super::StackHawkApi) super-trait combines all three.

mod auth;
mod listing;
mod scan_detail;

pub use auth::AuthApi;
pub use listing::ListingApi;
pub use scan_detail::ScanDetailApi;
