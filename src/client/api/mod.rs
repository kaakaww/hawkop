//! API trait definitions split by responsibility
//!
//! This module organizes the StackHawk API surface into focused sub-traits:
//! - [`AppApi`] - Application CRUD operations
//! - [`AuthApi`] - Authentication operations
//! - [`ListingApi`] - Collection listing operations
//! - [`ScanDetailApi`] - Scan drill-down operations
//! - [`TeamApi`] - Team CRUD operations
//! - [`PerchApi`] - Hosted scan control operations
//! - [`ConfigApi`] - Configuration management operations
//! - [`EnvironmentApi`] - Environment management operations
//! - [`OASApi`] - OpenAPI specification operations

mod app;
mod auth;
mod config;
mod env;
mod listing;
mod oas;
mod perch;
mod scan_detail;
mod team;

pub use app::AppApi;
pub use auth::AuthApi;
pub use config::ConfigApi;
pub use env::EnvironmentApi;
pub use listing::ListingApi;
pub use oas::OASApi;
pub use perch::PerchApi;
pub use scan_detail::ScanDetailApi;
pub use team::TeamApi;
