//! Shared CLI argument types
//!
//! This module contains reusable argument structs that can be flattened
//! into commands using `#[command(flatten)]`.

mod common;
mod filters;
mod pagination;

pub use common::{OutputFormat, SortDir};
pub use filters::{AuditFilterArgs, ScanFilterArgs};
pub use pagination::PaginationArgs;
