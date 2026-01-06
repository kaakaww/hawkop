//! Cache key generation using SHA-256 hashes

use sha2::{Digest, Sha256};

/// Generate a deterministic cache key from endpoint and parameters.
///
/// The key is a SHA-256 hash of the endpoint, org_id, and sorted parameters.
/// This ensures consistent keys regardless of parameter order.
pub fn cache_key(endpoint: &str, org_id: Option<&str>, params: &[(&str, &str)]) -> String {
    let mut hasher = Sha256::new();

    // Include endpoint
    hasher.update(endpoint.as_bytes());
    hasher.update(b"|");

    // Include org_id
    if let Some(org) = org_id {
        hasher.update(org.as_bytes());
    }
    hasher.update(b"|");

    // Sort and include params for deterministic key
    let mut sorted_params: Vec<_> = params.iter().collect();
    sorted_params.sort_by_key(|(k, _)| *k);

    for (k, v) in sorted_params {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
        hasher.update(b"&");
    }

    // Return hex-encoded hash
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let key1 = cache_key(
            "list_apps",
            Some("org-123"),
            &[("limit", "10"), ("page", "1")],
        );
        let key2 = cache_key(
            "list_apps",
            Some("org-123"),
            &[("page", "1"), ("limit", "10")],
        );

        // Same inputs in different order should produce same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_endpoints() {
        let key1 = cache_key("list_apps", Some("org-123"), &[]);
        let key2 = cache_key("list_scans", Some("org-123"), &[]);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_orgs() {
        let key1 = cache_key("list_apps", Some("org-123"), &[]);
        let key2 = cache_key("list_apps", Some("org-456"), &[]);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_no_org() {
        let key1 = cache_key("list_orgs", None, &[]);
        let key2 = cache_key("list_orgs", None, &[]);

        assert_eq!(key1, key2);
    }
}
