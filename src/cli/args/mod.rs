//! Shared CLI argument types
//!
//! This module contains reusable argument structs that can be flattened
//! into commands using `#[command(flatten)]`.

mod common;
mod filters;
mod global;
mod pagination;

pub use common::{OutputFormat, SortDir};
pub use filters::{AuditFilterArgs, ScanFilterArgs};
pub use global::GlobalOptions;
pub use pagination::PaginationArgs;
