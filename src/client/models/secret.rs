//! Secret models

use serde::{Deserialize, Serialize};

/// User secret (name only - values are not returned by list API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    /// Secret name
    #[serde(default)]
    pub name: String,
}
