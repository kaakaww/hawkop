//! Cached wrapper for StackHawk API client
//!
//! Provides transparent caching for all API responses using SQLite storage.

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::cache::{CacheStorage, CacheTtl, cache_key};
use crate::client::models::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, AuditFilterParams, AuditRecord,
    JwtToken, OASAsset, OrgPolicy, Organization, Repository, ScanConfig, ScanResult, Secret,
    StackHawkPolicy, Team, User,
};
use crate::client::{PagedResponse, PaginationParams, ScanFilterParams, StackHawkApi};
use crate::error::Result;

/// Cached wrapper for any StackHawkApi implementation.
///
/// Provides transparent caching of API responses using SQLite storage.
/// Cache can be disabled via the `enabled` flag (for `--no-cache`).
/// The cache is wrapped in a Mutex for thread-safety.
pub struct CachedStackHawkClient<C: StackHawkApi> {
    inner: Arc<C>,
    cache: Option<Mutex<CacheStorage>>,
}

impl<C: StackHawkApi> CachedStackHawkClient<C> {
    /// Create a new cached client wrapper.
    ///
    /// # Arguments
    /// * `inner` - The underlying API client to wrap
    /// * `enabled` - Whether caching is enabled (false for --no-cache)
    pub fn new(inner: C, enabled: bool) -> Self {
        let cache = if enabled {
            CacheStorage::open().ok().map(Mutex::new)
        } else {
            None
        };
        Self {
            inner: Arc::new(inner),
            cache,
        }
    }

    /// Get the inner client (for operations not part of the trait, like set_jwt)
    #[allow(dead_code)]
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Try to get cached data
    fn get_cached<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let cache = self.cache.as_ref()?;
        let guard = cache.lock().ok()?;
        guard
            .get(key)
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_slice(&data).ok())
    }

    /// Store data in cache
    fn set_cached<T: Serialize>(
        &self,
        key: &str,
        data: &T,
        endpoint: &str,
        org_id: Option<&str>,
        ttl: Duration,
    ) {
        if let Some(ref cache) = self.cache
            && let Ok(guard) = cache.lock()
            && let Ok(json) = serde_json::to_vec(data)
        {
            let _ = guard.put(key, &json, endpoint, org_id, ttl);
        }
    }
}

/// Convert pagination params to cache key params
fn pagination_to_params(pagination: Option<&PaginationParams>) -> Vec<(&'static str, String)> {
    match pagination {
        Some(p) => {
            let mut params = vec![];
            if let Some(page) = p.page {
                params.push(("page", page.to_string()));
            }
            if let Some(page_size) = p.page_size {
                params.push(("page_size", page_size.to_string()));
            }
            if let Some(ref sort_by) = p.sort_by {
                params.push(("sort_by", sort_by.clone()));
            }
            params
        }
        None => vec![],
    }
}

/// Convert scan filter params to cache key params
fn scan_filters_to_params(filters: Option<&ScanFilterParams>) -> Vec<(&'static str, String)> {
    match filters {
        Some(f) => {
            let mut params = vec![];
            if !f.app_ids.is_empty() {
                params.push(("app_ids", f.app_ids.join(",")));
            }
            if !f.envs.is_empty() {
                params.push(("envs", f.envs.join(",")));
            }
            if !f.team_ids.is_empty() {
                params.push(("team_ids", f.team_ids.join(",")));
            }
            if let Some(start) = f.start {
                params.push(("start", start.to_string()));
            }
            if let Some(end) = f.end {
                params.push(("end", end.to_string()));
            }
            params
        }
        None => vec![],
    }
}

/// Convert audit filter params to cache key params
fn audit_filters_to_params(filters: Option<&AuditFilterParams>) -> Vec<(&'static str, String)> {
    match filters {
        Some(f) => {
            let mut params = vec![];
            if !f.types.is_empty() {
                params.push(("types", f.types.join(",")));
            }
            if !f.org_types.is_empty() {
                params.push(("org_types", f.org_types.join(",")));
            }
            if let Some(ref name) = f.name {
                params.push(("name", name.clone()));
            }
            if let Some(ref email) = f.email {
                params.push(("email", email.clone()));
            }
            if let Some(start) = f.start {
                params.push(("start", start.to_string()));
            }
            if let Some(end) = f.end {
                params.push(("end", end.to_string()));
            }
            if let Some(size) = f.page_size {
                params.push(("page_size", size.to_string()));
            }
            if let Some(ref token) = f.page_token {
                params.push(("page_token", token.clone()));
            }
            params
        }
        None => vec![],
    }
}

#[async_trait]
impl<C: StackHawkApi + 'static> StackHawkApi for CachedStackHawkClient<C> {
    /// Authenticate - NEVER cached (security sensitive)
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken> {
        self.inner.authenticate(api_key).await
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        let key = cache_key("list_orgs", None, &[]);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_orgs");
            return Ok(cached);
        }

        let result = self.inner.list_orgs().await?;
        self.set_cached(&key, &result, "list_orgs", None, CacheTtl::ORGS);
        Ok(result)
    }

    async fn list_apps(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Application>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_apps", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_apps");
            return Ok(cached);
        }

        let result = self.inner.list_apps(org_id, pagination).await?;
        self.set_cached(&key, &result, "list_apps", Some(org_id), CacheTtl::APPS);
        Ok(result)
    }

    async fn list_apps_paged(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<Application>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_apps_paged", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_apps_paged");
            return Ok(cached);
        }

        let result = self.inner.list_apps_paged(org_id, pagination).await?;
        self.set_cached(
            &key,
            &result,
            "list_apps_paged",
            Some(org_id),
            CacheTtl::APPS,
        );
        Ok(result)
    }

    async fn list_scans(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
        filters: Option<&ScanFilterParams>,
    ) -> Result<Vec<ScanResult>> {
        let mut params = pagination_to_params(pagination);
        params.extend(scan_filters_to_params(filters));
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_scans", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_scans");
            return Ok(cached);
        }

        let result = self.inner.list_scans(org_id, pagination, filters).await?;
        self.set_cached(
            &key,
            &result,
            "list_scans",
            Some(org_id),
            CacheTtl::SCAN_LIST,
        );
        Ok(result)
    }

    async fn list_scans_paged(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
        filters: Option<&ScanFilterParams>,
    ) -> Result<PagedResponse<ScanResult>> {
        let mut params = pagination_to_params(pagination);
        params.extend(scan_filters_to_params(filters));
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_scans_paged", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_scans_paged");
            return Ok(cached);
        }

        let result = self
            .inner
            .list_scans_paged(org_id, pagination, filters)
            .await?;
        self.set_cached(
            &key,
            &result,
            "list_scans_paged",
            Some(org_id),
            CacheTtl::SCAN_LIST,
        );
        Ok(result)
    }

    async fn list_users(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<User>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_users", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_users");
            return Ok(cached);
        }

        let result = self.inner.list_users(org_id, pagination).await?;
        self.set_cached(&key, &result, "list_users", Some(org_id), CacheTtl::USERS);
        Ok(result)
    }

    async fn list_teams(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Team>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_teams", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_teams");
            return Ok(cached);
        }

        let result = self.inner.list_teams(org_id, pagination).await?;
        self.set_cached(&key, &result, "list_teams", Some(org_id), CacheTtl::TEAMS);
        Ok(result)
    }

    async fn list_stackhawk_policies(&self) -> Result<Vec<StackHawkPolicy>> {
        let key = cache_key("list_stackhawk_policies", None, &[]);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_stackhawk_policies");
            return Ok(cached);
        }

        let result = self.inner.list_stackhawk_policies().await?;
        self.set_cached(
            &key,
            &result,
            "list_stackhawk_policies",
            None,
            CacheTtl::POLICIES,
        );
        Ok(result)
    }

    async fn list_org_policies(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OrgPolicy>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_org_policies", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_org_policies");
            return Ok(cached);
        }

        let result = self.inner.list_org_policies(org_id, pagination).await?;
        self.set_cached(
            &key,
            &result,
            "list_org_policies",
            Some(org_id),
            CacheTtl::POLICIES,
        );
        Ok(result)
    }

    async fn list_repos(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Repository>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_repos", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_repos");
            return Ok(cached);
        }

        let result = self.inner.list_repos(org_id, pagination).await?;
        self.set_cached(&key, &result, "list_repos", Some(org_id), CacheTtl::REPOS);
        Ok(result)
    }

    async fn list_oas(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OASAsset>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_oas", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_oas");
            return Ok(cached);
        }

        let result = self.inner.list_oas(org_id, pagination).await?;
        self.set_cached(&key, &result, "list_oas", Some(org_id), CacheTtl::OAS);
        Ok(result)
    }

    async fn list_scan_configs(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ScanConfig>> {
        let params = pagination_to_params(pagination);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_scan_configs", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_scan_configs");
            return Ok(cached);
        }

        let result = self.inner.list_scan_configs(org_id, pagination).await?;
        self.set_cached(
            &key,
            &result,
            "list_scan_configs",
            Some(org_id),
            CacheTtl::SCAN_CONFIGS,
        );
        Ok(result)
    }

    async fn list_secrets(&self) -> Result<Vec<Secret>> {
        let key = cache_key("list_secrets", None, &[]);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_secrets");
            return Ok(cached);
        }

        let result = self.inner.list_secrets().await?;
        self.set_cached(&key, &result, "list_secrets", None, CacheTtl::SECRETS);
        Ok(result)
    }

    async fn list_audit(
        &self,
        org_id: &str,
        filters: Option<&AuditFilterParams>,
    ) -> Result<Vec<AuditRecord>> {
        let params = audit_filters_to_params(filters);
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_audit", Some(org_id), &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_audit");
            return Ok(cached);
        }

        let result = self.inner.list_audit(org_id, filters).await?;
        self.set_cached(&key, &result, "list_audit", Some(org_id), CacheTtl::AUDIT);
        Ok(result)
    }

    async fn get_scan(&self, org_id: &str, scan_id: &str) -> Result<ScanResult> {
        let key = cache_key("get_scan", Some(org_id), &[("scan_id", scan_id)]);

        if let Some(cached) = self.get_cached::<ScanResult>(&key) {
            log::debug!("Cache hit: get_scan");
            return Ok(cached);
        }

        let result = self.inner.get_scan(org_id, scan_id).await?;

        // TTL depends on scan status
        let ttl = match result.scan.status.to_uppercase().as_str() {
            "COMPLETED" => CacheTtl::SCAN_DETAIL_COMPLETED,
            "STARTED" | "RUNNING" | "PENDING" => CacheTtl::SCAN_DETAIL_RUNNING,
            _ => CacheTtl::SCAN_LIST,
        };

        self.set_cached(&key, &result, "get_scan", Some(org_id), ttl);
        Ok(result)
    }

    async fn list_scan_alerts(
        &self,
        scan_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ApplicationAlert>> {
        let mut params = pagination_to_params(pagination);
        params.push(("scan_id", scan_id.to_string()));
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("list_scan_alerts", None, &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: list_scan_alerts");
            return Ok(cached);
        }

        let result = self.inner.list_scan_alerts(scan_id, pagination).await?;
        self.set_cached(&key, &result, "list_scan_alerts", None, CacheTtl::ALERTS);
        Ok(result)
    }

    async fn get_alert_with_paths(
        &self,
        scan_id: &str,
        plugin_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<AlertResponse> {
        let mut params = pagination_to_params(pagination);
        params.push(("scan_id", scan_id.to_string()));
        params.push(("plugin_id", plugin_id.to_string()));
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let key = cache_key("get_alert_with_paths", None, &params_ref);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: get_alert_with_paths");
            return Ok(cached);
        }

        let result = self
            .inner
            .get_alert_with_paths(scan_id, plugin_id, pagination)
            .await?;
        self.set_cached(
            &key,
            &result,
            "get_alert_with_paths",
            None,
            CacheTtl::ALERT_PATHS,
        );
        Ok(result)
    }

    async fn get_alert_message(
        &self,
        scan_id: &str,
        alert_uri_id: &str,
        message_id: &str,
        include_curl: bool,
    ) -> Result<AlertMsgResponse> {
        let params = [
            ("scan_id", scan_id),
            ("alert_uri_id", alert_uri_id),
            ("message_id", message_id),
            ("include_curl", if include_curl { "true" } else { "false" }),
        ];
        let key = cache_key("get_alert_message", None, &params);

        if let Some(cached) = self.get_cached(&key) {
            log::debug!("Cache hit: get_alert_message");
            return Ok(cached);
        }

        let result = self
            .inner
            .get_alert_message(scan_id, alert_uri_id, message_id, include_curl)
            .await?;

        // Alert messages are immutable - cache for 24 hours
        self.set_cached(
            &key,
            &result,
            "get_alert_message",
            None,
            CacheTtl::SCAN_DETAIL_COMPLETED,
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockStackHawkClient;
    use tempfile::TempDir;

    fn create_test_client(enabled: bool) -> (CachedStackHawkClient<MockStackHawkClient>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mock = MockStackHawkClient::new();

        // Create cache in temp dir (wrapped in Mutex for thread safety)
        let cache = if enabled {
            CacheStorage::open_at(temp_dir.path()).ok().map(Mutex::new)
        } else {
            None
        };

        let client = CachedStackHawkClient {
            inner: Arc::new(mock),
            cache,
        };

        (client, temp_dir)
    }

    #[tokio::test]
    async fn test_authenticate_never_cached() {
        let (client, _dir) = create_test_client(true);

        // First call
        let result1 = client.authenticate("test-key").await;
        assert!(result1.is_ok());

        // Second call should also hit the API (not cached)
        let result2 = client.authenticate("test-key").await;
        assert!(result2.is_ok());

        // Both calls should have gone to inner client
        let counts = client.inner.call_counts().await;
        assert_eq!(counts.authenticate, 2);
    }

    #[tokio::test]
    async fn test_cache_disabled_bypasses_cache() {
        let (client, _dir) = create_test_client(false);

        // First call
        let _ = client.list_orgs().await;

        // Second call should also hit API
        let _ = client.list_orgs().await;

        // Both calls should have gone to inner client
        let counts = client.inner.call_counts().await;
        assert_eq!(counts.list_orgs, 2);
    }

    #[tokio::test]
    async fn test_list_orgs_cached() {
        let (client, _dir) = create_test_client(true);

        // First call - cache miss
        let result1 = client.list_orgs().await.unwrap();

        // Second call - cache hit
        let result2 = client.list_orgs().await.unwrap();

        // Results should be the same
        assert_eq!(result1.len(), result2.len());

        // Only first call should have gone to inner client
        let counts = client.inner.call_counts().await;
        assert_eq!(counts.list_orgs, 1);
    }
}
