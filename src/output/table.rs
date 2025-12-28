//! Table output formatting

use tabled::{
    Table, Tabled,
    settings::Style,
};

/// Format data as a table with clean minimal style
pub fn format_table<T: Tabled>(data: &[T]) -> String {
    if data.is_empty() {
        return "No results found.".to_string();
    }

    let mut table = Table::new(data);

    // Clean minimal style: vertical separators, thin header underline, no outer borders
    table.with(
        Style::blank()
            .vertical('│')
            .horizontals([(1, tabled::settings::style::HorizontalLine::new('─').intersection('┼'))])
    );

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Tabled)]
    struct TestRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "NAME")]
        name: String,
    }

    #[test]
    fn test_format_table_empty() {
        let items: Vec<TestRow> = vec![];
        let result = format_table(&items);
        assert_eq!(result, "No results found.");
    }

    #[test]
    fn test_format_table_single_row() {
        let items = vec![TestRow {
            id: "123".to_string(),
            name: "Test".to_string(),
        }];

        let result = format_table(&items);

        assert!(result.contains("ID"));
        assert!(result.contains("NAME"));
        assert!(result.contains("123"));
        assert!(result.contains("Test"));
    }

    #[test]
    fn test_format_table_multiple_rows() {
        let items = vec![
            TestRow {
                id: "1".to_string(),
                name: "First".to_string(),
            },
            TestRow {
                id: "2".to_string(),
                name: "Second".to_string(),
            },
        ];

        let result = format_table(&items);

        assert!(result.contains("First"));
        assert!(result.contains("Second"));
    }

    #[test]
    fn test_format_table_uses_minimal_style() {
        let items = vec![TestRow {
            id: "1".to_string(),
            name: "Test".to_string(),
        }];

        let result = format_table(&items);

        // Minimal style uses │ for vertical separators and ─ for header underline
        assert!(result.contains("│"));
        assert!(result.contains("─"));
        // No outer borders
        assert!(!result.contains("╭"));
        assert!(!result.contains("╰"));
    }
}
