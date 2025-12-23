//! Output formatting for CLI results
//!
//! This module provides a unified formatting abstraction for CLI output,
//! supporting both table and JSON formats.

use serde::Serialize;
use tabled::Tabled;

use crate::cli::OutputFormat;
use crate::error::Result;

pub mod json;
pub mod table;

/// Trait for types that can be formatted for output.
///
/// Implementors can format themselves according to the specified output format.
pub trait Formattable {
    /// Format the data according to the specified format.
    fn format(&self, format: OutputFormat) -> Result<String>;

    /// Format and print to stdout.
    fn print(&self, format: OutputFormat) -> Result<()> {
        let output = self.format(format)?;
        println!("{}", output);
        Ok(())
    }
}

/// Blanket implementation for slices of types that implement Tabled and Serialize.
///
/// This allows any `Vec<T>` or `&[T]` where T implements both traits to be
/// automatically formatted as either a table or JSON.
impl<T> Formattable for [T]
where
    T: Tabled + Serialize,
{
    fn format(&self, format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Table => Ok(table::format_table(self)),
            OutputFormat::Json => json::format_json(self).map_err(|e| {
                crate::error::Error::Other(format!("JSON serialization failed: {}", e))
            }),
        }
    }
}

/// Blanket implementation for Vec<T> delegating to slice implementation.
impl<T> Formattable for Vec<T>
where
    T: Tabled + Serialize,
{
    fn format(&self, format: OutputFormat) -> Result<String> {
        self.as_slice().format(format)
    }
}
