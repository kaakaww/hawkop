//! Output formatting for CLI results

use crate::cli::OutputFormat;
use crate::error::Result;

pub mod json;
pub mod table;

/// Trait for types that can be formatted for output
#[allow(dead_code)]
pub trait Formattable {
    /// Format the data according to the specified format
    fn format(&self, format: OutputFormat) -> Result<String>;
}

/// Format and print data to stdout
#[allow(dead_code)]
pub fn print<T: Formattable>(data: &T, format: OutputFormat) -> Result<()> {
    let output = data.format(format)?;
    println!("{}", output);
    Ok(())
}
