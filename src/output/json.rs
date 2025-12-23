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
