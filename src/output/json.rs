//! JSON output formatting

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Wrapper for JSON output with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput<T> {
    /// The actual data
    pub data: T,

    /// Metadata about the response
    pub meta: Metadata,
}

/// Metadata included in JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// Timestamp of the response
    pub timestamp: String,

    /// CLI version
    pub version: String,
}

impl<T> JsonOutput<T> {
    /// Create a new JSON output with metadata
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: Metadata {
                timestamp: Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

/// Format data as pretty-printed JSON
pub fn format_json<T: Serialize + ?Sized>(data: &T) -> Result<String, serde_json::Error> {
    let output = JsonOutput::new(data);
    serde_json::to_string_pretty(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Clone)]
    struct TestItem {
        id: String,
        name: String,
    }

    #[test]
    fn test_json_output_new() {
        let data = vec!["item1", "item2"];
        let output = JsonOutput::new(data);

        assert_eq!(output.data, vec!["item1", "item2"]);
        assert_eq!(output.meta.version, env!("CARGO_PKG_VERSION"));
        assert!(!output.meta.timestamp.is_empty());
    }

    #[test]
    fn test_format_json_basic() {
        let items = vec![TestItem {
            id: "1".to_string(),
            name: "Test".to_string(),
        }];

        let result = format_json(&items).unwrap();

        assert!(result.contains("\"data\""));
        assert!(result.contains("\"meta\""));
        assert!(result.contains("\"id\": \"1\""));
        assert!(result.contains("\"name\": \"Test\""));
        assert!(result.contains("\"timestamp\""));
        assert!(result.contains("\"version\""));
    }

    #[test]
    fn test_format_json_empty_vec() {
        let items: Vec<TestItem> = vec![];
        let result = format_json(&items).unwrap();

        assert!(result.contains("\"data\": []"));
    }

    #[test]
    fn test_format_json_multiple_items() {
        let items = vec![
            TestItem {
                id: "1".to_string(),
                name: "First".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                name: "Second".to_string(),
            },
        ];

        let result = format_json(&items).unwrap();

        assert!(result.contains("\"First\""));
        assert!(result.contains("\"Second\""));
    }
}
