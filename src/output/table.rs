//! Table output formatting

use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Rows},
};

/// Format data as a table
pub fn format_table<T: Tabled>(data: &[T]) -> String {
    if data.is_empty() {
        return "No results found.".to_string();
    }

    let mut table = Table::new(data);
    table
        .with(Style::rounded())
        .with(Modify::new(Rows::first()).with(Alignment::center()));

    table.to_string()
}
