//! Display model implementations for table and JSON output
//!
//! Display models transform API response types into CLI-friendly formats
//! with appropriate column names and serialization.

mod app;
mod audit;
mod common;
mod config;
mod finding;
mod oas;
mod org;
mod policy;
mod repo;
mod scan;
mod secret;
mod user;

// Re-export all display types used by CLI commands
pub use app::AppDisplay;
pub use audit::AuditDisplay;
pub use config::ConfigDisplay;
pub use finding::{AlertDetail, AlertFindingDisplay, AlertMessageDetail, PrettyAlertDisplay};
pub use oas::OASDisplay;
pub use org::OrgDisplay;
pub use policy::PolicyDisplay;
pub use repo::RepoDisplay;
pub use scan::ScanDisplay;
pub use secret::SecretDisplay;
pub use user::{TeamListDisplay, UserDisplay};
