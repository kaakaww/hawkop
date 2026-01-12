//! Secret display model

use serde::Serialize;
use tabled::Tabled;

use crate::client::models::Secret;

/// Secret display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct SecretDisplay {
    /// Secret name
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<Secret> for SecretDisplay {
    fn from(secret: Secret) -> Self {
        Self { name: secret.name }
    }
}

impl From<&Secret> for SecretDisplay {
    fn from(secret: &Secret) -> Self {
        SecretDisplay::from(secret.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_display_from_secret() {
        let secret = Secret {
            name: "API_KEY".to_string(),
        };

        let display = SecretDisplay::from(secret);

        assert_eq!(display.name, "API_KEY");
    }
}
