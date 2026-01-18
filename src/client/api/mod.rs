//! API trait definitions split by responsibility
//!
//! This module organizes the StackHawk API surface into focused sub-traits:
//! - [`AuthApi`] - Authentication operations
//! - [`ListingApi`] - Collection listing operations
//! - [`ScanDetailApi`] - Scan drill-down operations
//! - [`TeamApi`] - Team CRUD operations

mod auth;
mod listing;
mod scan_detail;
mod team;

pub use auth::AuthApi;
pub use listing::ListingApi;
pub use scan_detail::ScanDetailApi;
pub use team::TeamApi;
