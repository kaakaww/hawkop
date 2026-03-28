//! Scan detail API trait for drill-down operations

use async_trait::async_trait;

use crate::client::models::{
    AlertMsgResponse, AlertResponse, ApplicationAlert, CurrentFindingsResponse, ScanResult,
};
use crate::client::pagination::PaginationParams;
use crate::error::Result;

/// Scan drill-down operations for the StackHawk API
///
/// This trait covers operations for exploring scan results in detail:
/// fetching individual scans, listing alerts, and examining specific findings.
#[async_trait]
pub trait ScanDetailApi: Send + Sync {
    /// Get a single scan by ID
    ///
    /// Fetches detailed scan information including alert stats.
    async fn get_scan(&self, org_id: &str, scan_id: &str) -> Result<ScanResult>;

    /// List all alerts (plugins) for a scan
    ///
    /// Returns plugin-level finding summaries with counts and triage stats.
    async fn list_scan_alerts(
        &self,
        scan_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ApplicationAlert>>;

    /// Get alert details with affected paths for a specific plugin
    ///
    /// Returns the alert info plus paginated list of vulnerable endpoints.
    async fn get_alert_with_paths(
        &self,
        scan_id: &str,
        plugin_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<AlertResponse>;

    /// Get HTTP request/response details for a specific finding
    ///
    /// Returns full message details including optional curl validation command.
    async fn get_alert_message(
        &self,
        scan_id: &str,
        alert_uri_id: &str,
        message_id: &str,
        include_curl: bool,
    ) -> Result<AlertMsgResponse>;

    /// List organization findings from the reports endpoint
    ///
    /// Returns enriched finding data including remediation advice, first/last seen
    /// timestamps, and stable finding hashes. Supports filtering by app IDs.
    ///
    /// Used by `scan get --detail full` for enrichment and future `findings list` command.
    async fn list_org_findings(
        &self,
        org_id: &str,
        app_ids: &[String],
        page_size: Option<usize>,
        page_token: Option<usize>,
    ) -> Result<CurrentFindingsResponse>;
}
