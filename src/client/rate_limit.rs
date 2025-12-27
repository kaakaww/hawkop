//! Per-endpoint rate limiting for StackHawk API
//!
//! Implements reactive rate limiting that only activates after receiving a 429.
//! Different endpoint patterns have different rate limits.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use log::debug;
use tokio::sync::RwLock;

/// Categories of API endpoints with their rate limits.
///
/// Rate limits are based on StackHawk API documentation:
/// - Scan endpoints: 4800/min (80/sec)
/// - User endpoint: 4800/min (80/sec)
/// - App list/org endpoints: 4800/min (80/sec)
/// - Org invite endpoints: 10/min (0.17/sec)
/// - All other endpoints: 360/min (6/sec)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointCategory {
    /// /api/v1/scan.* - 80 req/sec (4800/min)
    Scan,
    /// /api/v1/user (GET) - 80 req/sec
    User,
    /// /api/v1/app/.*/list (GET) - 80 req/sec
    AppList,
    /// /api/v1/app/.*/org (GET) - 80 req/sec
    AppOrg,
    /// /api/v1/org/.*/invite.* (POST) - 0.17 req/sec (10/min)
    OrgInvite,
    /// Default for all other endpoints - 6 req/sec (360/min)
    Default,
}

impl EndpointCategory {
    /// All endpoint categories for initialization.
    pub const ALL: [EndpointCategory; 6] = [
        EndpointCategory::Scan,
        EndpointCategory::User,
        EndpointCategory::AppList,
        EndpointCategory::AppOrg,
        EndpointCategory::OrgInvite,
        EndpointCategory::Default,
    ];

    /// Categorize a request based on path and method.
    ///
    /// The path should be the API path without the base URL (e.g., "/scan/org-123").
    pub fn from_request(path: &str, method: &reqwest::Method) -> Self {
        // Strip API version prefix if present
        let path = path
            .strip_prefix("/api/v1")
            .or_else(|| path.strip_prefix("/api/v2"))
            .unwrap_or(path);

        // Match patterns in order of specificity
        if path.starts_with("/scan") {
            return EndpointCategory::Scan;
        }

        if path == "/user" && *method == reqwest::Method::GET {
            return EndpointCategory::User;
        }

        // Pattern: /app/{id}/list
        if path.starts_with("/app/") && path.ends_with("/list") && *method == reqwest::Method::GET {
            return EndpointCategory::AppList;
        }

        // Pattern: /app/{id}/org
        if path.starts_with("/app/") && path.ends_with("/org") && *method == reqwest::Method::GET {
            return EndpointCategory::AppOrg;
        }

        // Pattern: /org/{id}/invite*
        if path.contains("/org/") && path.contains("/invite") && *method == reqwest::Method::POST {
            return EndpointCategory::OrgInvite;
        }

        EndpointCategory::Default
    }

    /// Get the rate limit for this category (requests per second).
    pub fn rate_limit(&self) -> f64 {
        match self {
            EndpointCategory::Scan => 80.0,
            EndpointCategory::User => 80.0,
            EndpointCategory::AppList => 80.0,
            EndpointCategory::AppOrg => 80.0,
            EndpointCategory::OrgInvite => 0.167, // 10 per minute
            EndpointCategory::Default => 6.0,
        }
    }
}

/// Rate limiter state for a single endpoint category.
pub struct EndpointRateLimiter {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    active: AtomicBool,
    category: EndpointCategory,
}

impl EndpointRateLimiter {
    /// Create a new rate limiter for an endpoint category.
    pub fn new(category: EndpointCategory) -> Self {
        let rate = category.rate_limit();

        // Handle sub-1 rates by using per-minute quotas
        let quota = if rate >= 1.0 {
            Quota::per_second(NonZeroU32::new(rate as u32).unwrap_or(NonZeroU32::MIN))
        } else {
            // For 0.167 req/sec = 10/min, use per_minute
            let per_min = (rate * 60.0).round() as u32;
            Quota::per_minute(NonZeroU32::new(per_min).unwrap_or(NonZeroU32::MIN))
        };

        Self {
            limiter: RateLimiter::direct(quota),
            active: AtomicBool::new(false),
            category,
        }
    }

    /// Activate rate limiting for this category.
    pub fn activate(&self) {
        let was_active = self.active.swap(true, Ordering::SeqCst);
        if !was_active {
            debug!("Rate limiting activated for {:?}", self.category);
        }
    }

    /// Check if rate limiting is active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Wait for permission if rate limiting is active.
    pub async fn wait_if_active(&self) {
        if self.is_active() {
            debug!("Waiting for rate limiter {:?}", self.category);
            self.limiter.until_ready().await;
        }
    }
}

/// Collection of rate limiters for all endpoint categories.
pub struct RateLimiterSet {
    limiters: RwLock<HashMap<EndpointCategory, EndpointRateLimiter>>,
}

impl Default for RateLimiterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterSet {
    /// Create a new set of rate limiters for all endpoint categories.
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Pre-create limiters for all categories
        for category in EndpointCategory::ALL {
            map.insert(category, EndpointRateLimiter::new(category));
        }

        Self {
            limiters: RwLock::new(map),
        }
    }

    /// Wait for rate limit permission for a category (if active).
    pub async fn wait_for(&self, category: EndpointCategory) {
        let limiters = self.limiters.read().await;
        if let Some(limiter) = limiters.get(&category) {
            limiter.wait_if_active().await;
        }
    }

    /// Activate rate limiting for a category (called on 429).
    pub async fn activate(&self, category: EndpointCategory) {
        let limiters = self.limiters.read().await;
        if let Some(limiter) = limiters.get(&category) {
            limiter.activate();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_endpoints() {
        assert_eq!(
            EndpointCategory::from_request("/scan/org-123", &reqwest::Method::GET),
            EndpointCategory::Scan
        );
        assert_eq!(
            EndpointCategory::from_request("/api/v1/scan/org-123", &reqwest::Method::GET),
            EndpointCategory::Scan
        );
        assert_eq!(
            EndpointCategory::from_request("/scan/org-123/details", &reqwest::Method::GET),
            EndpointCategory::Scan
        );
    }

    #[test]
    fn test_user_endpoint() {
        assert_eq!(
            EndpointCategory::from_request("/user", &reqwest::Method::GET),
            EndpointCategory::User
        );
        assert_eq!(
            EndpointCategory::from_request("/api/v1/user", &reqwest::Method::GET),
            EndpointCategory::User
        );
        // POST to /user should be default
        assert_eq!(
            EndpointCategory::from_request("/user", &reqwest::Method::POST),
            EndpointCategory::Default
        );
    }

    #[test]
    fn test_app_list_endpoints() {
        assert_eq!(
            EndpointCategory::from_request("/app/app-123/list", &reqwest::Method::GET),
            EndpointCategory::AppList
        );
        assert_eq!(
            EndpointCategory::from_request("/api/v1/app/app-123/list", &reqwest::Method::GET),
            EndpointCategory::AppList
        );
        // POST should be default
        assert_eq!(
            EndpointCategory::from_request("/app/app-123/list", &reqwest::Method::POST),
            EndpointCategory::Default
        );
    }

    #[test]
    fn test_app_org_endpoints() {
        assert_eq!(
            EndpointCategory::from_request("/app/app-123/org", &reqwest::Method::GET),
            EndpointCategory::AppOrg
        );
        assert_eq!(
            EndpointCategory::from_request("/api/v1/app/app-123/org", &reqwest::Method::GET),
            EndpointCategory::AppOrg
        );
    }

    #[test]
    fn test_org_invite_endpoints() {
        assert_eq!(
            EndpointCategory::from_request("/org/org-123/invite", &reqwest::Method::POST),
            EndpointCategory::OrgInvite
        );
        assert_eq!(
            EndpointCategory::from_request("/org/org-123/invite/bulk", &reqwest::Method::POST),
            EndpointCategory::OrgInvite
        );
        // GET should be default
        assert_eq!(
            EndpointCategory::from_request("/org/org-123/invite", &reqwest::Method::GET),
            EndpointCategory::Default
        );
    }

    #[test]
    fn test_default_endpoints() {
        assert_eq!(
            EndpointCategory::from_request("/auth/login", &reqwest::Method::GET),
            EndpointCategory::Default
        );
        assert_eq!(
            EndpointCategory::from_request("/org/org-123/apps", &reqwest::Method::GET),
            EndpointCategory::Default
        );
        assert_eq!(
            EndpointCategory::from_request("/unknown/path", &reqwest::Method::GET),
            EndpointCategory::Default
        );
    }

    #[test]
    fn test_rate_limits() {
        assert_eq!(EndpointCategory::Scan.rate_limit(), 80.0);
        assert_eq!(EndpointCategory::User.rate_limit(), 80.0);
        assert_eq!(EndpointCategory::AppList.rate_limit(), 80.0);
        assert_eq!(EndpointCategory::AppOrg.rate_limit(), 80.0);
        assert_eq!(EndpointCategory::OrgInvite.rate_limit(), 0.167);
        assert_eq!(EndpointCategory::Default.rate_limit(), 6.0);
    }

    #[test]
    fn test_endpoint_rate_limiter_activation() {
        let limiter = EndpointRateLimiter::new(EndpointCategory::Scan);
        assert!(!limiter.is_active());

        limiter.activate();
        assert!(limiter.is_active());

        // Second activation should be idempotent
        limiter.activate();
        assert!(limiter.is_active());
    }

    #[tokio::test]
    async fn test_rate_limiter_set_creation() {
        let set = RateLimiterSet::new();
        let limiters = set.limiters.read().await;

        // All categories should be present
        for category in EndpointCategory::ALL {
            assert!(limiters.contains_key(&category));
        }
    }
}
