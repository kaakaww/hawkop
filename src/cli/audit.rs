//! Audit log management commands

use chrono::{Duration, Utc};

use crate::cli::args::GlobalOptions;
use crate::cli::{AuditFilterArgs, CommandContext, SortDir};
use crate::client::ListingApi;
use crate::client::models::AuditFilterParams;
use crate::error::Result;
use crate::models::AuditDisplay;
use crate::output::Formattable;

/// Run the audit list command
pub async fn list(opts: &GlobalOptions, filters: &AuditFilterArgs) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;

    // Convert CLI args to API filter params
    let api_filters = build_filter_params(filters)?;

    let org_id = ctx.require_org_id()?;
    let records = ctx.client.list_audit(org_id, Some(&api_filters)).await?;

    let display_records: Vec<AuditDisplay> = records.into_iter().map(AuditDisplay::from).collect();
    display_records.print(ctx.format)?;

    Ok(())
}

/// Convert CLI filter args to API filter params
fn build_filter_params(args: &AuditFilterArgs) -> Result<AuditFilterParams> {
    let mut params = AuditFilterParams::new();

    // Activity types
    if !args.activity_type.is_empty() {
        params.types = args.activity_type.clone();
    }

    // Org activity types
    if !args.org_type.is_empty() {
        params.org_types = args.org_type.clone();
    }

    // User name filter
    if let Some(ref user) = args.user {
        params.name = Some(user.clone());
    }

    // Email filter
    if let Some(ref email) = args.email {
        params.email = Some(email.clone());
    }

    // Parse start date (since)
    if let Some(ref since) = args.since {
        params.start = Some(parse_date_to_millis(since)?);
    }

    // Parse end date (until)
    if let Some(ref until) = args.until {
        params.end = Some(parse_date_to_millis(until)?);
    }

    // Sort direction
    params.sort_dir = Some(match args.sort_dir {
        SortDir::Asc => "asc".to_string(),
        SortDir::Desc => "desc".to_string(),
    });

    // Page size / limit
    if let Some(limit) = args.limit {
        params.page_size = Some(limit.min(1000)); // API max is 1000
    } else {
        params.page_size = Some(100); // Default
    }

    Ok(params)
}

/// Parse date string to milliseconds timestamp.
///
/// Supports:
/// - Relative: "7d" (7 days ago), "30d" (30 days ago), "1w" (1 week ago)
/// - ISO date: "2024-01-15"
/// - ISO datetime: "2024-01-15T10:30:00Z"
fn parse_date_to_millis(date_str: &str) -> Result<i64> {
    let now = Utc::now();

    // Try relative format first (e.g., "7d", "30d", "1w")
    if let Some(stripped) = date_str.strip_suffix('d')
        && let Ok(days) = stripped.parse::<i64>()
    {
        let target = now - Duration::days(days);
        return Ok(target.timestamp_millis());
    }

    if let Some(stripped) = date_str.strip_suffix('w')
        && let Ok(weeks) = stripped.parse::<i64>()
    {
        let target = now - Duration::weeks(weeks);
        return Ok(target.timestamp_millis());
    }

    if let Some(stripped) = date_str.strip_suffix('h')
        && let Ok(hours) = stripped.parse::<i64>()
    {
        let target = now - Duration::hours(hours);
        return Ok(target.timestamp_millis());
    }

    // Try ISO date format (YYYY-MM-DD)
    if date_str.len() == 10
        && date_str.chars().nth(4) == Some('-')
        && let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
    {
        let datetime = date.and_hms_opt(0, 0, 0).expect("valid time").and_utc();
        return Ok(datetime.timestamp_millis());
    }

    // Try ISO datetime format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.timestamp_millis());
    }

    // Try without timezone (assume UTC)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc().timestamp_millis());
    }

    Err(crate::error::Error::Other(format!(
        "Invalid date format: '{}'. Use relative (7d, 30d, 1w) or ISO format (YYYY-MM-DD)",
        date_str
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ========================================================================
    // parse_date_to_millis tests
    // ========================================================================

    #[test]
    fn test_parse_date_relative_days() {
        let now = Utc::now();

        // 7 days ago
        let result = parse_date_to_millis("7d").unwrap();
        let expected = (now - Duration::days(7)).timestamp_millis();
        // Allow 1 second tolerance for test timing
        assert!((result - expected).abs() < 1000);

        // 30 days ago
        let result = parse_date_to_millis("30d").unwrap();
        let expected = (now - Duration::days(30)).timestamp_millis();
        assert!((result - expected).abs() < 1000);
    }

    #[test]
    fn test_parse_date_relative_weeks() {
        let now = Utc::now();

        let result = parse_date_to_millis("1w").unwrap();
        let expected = (now - Duration::weeks(1)).timestamp_millis();
        assert!((result - expected).abs() < 1000);

        let result = parse_date_to_millis("4w").unwrap();
        let expected = (now - Duration::weeks(4)).timestamp_millis();
        assert!((result - expected).abs() < 1000);
    }

    #[test]
    fn test_parse_date_relative_hours() {
        let now = Utc::now();

        let result = parse_date_to_millis("24h").unwrap();
        let expected = (now - Duration::hours(24)).timestamp_millis();
        assert!((result - expected).abs() < 1000);
    }

    #[test]
    fn test_parse_date_iso_date() {
        let result = parse_date_to_millis("2024-01-15").unwrap();
        let expected = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        assert_eq!(result, expected.timestamp_millis());
    }

    #[test]
    fn test_parse_date_iso_datetime_with_tz() {
        let result = parse_date_to_millis("2024-01-15T10:30:00Z").unwrap();
        let expected = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        assert_eq!(result, expected.timestamp_millis());
    }

    #[test]
    fn test_parse_date_iso_datetime_without_tz() {
        let result = parse_date_to_millis("2024-01-15T10:30:00").unwrap();
        let expected = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        assert_eq!(result, expected.timestamp_millis());
    }

    #[test]
    fn test_parse_date_invalid_format() {
        let result = parse_date_to_millis("invalid");
        assert!(result.is_err());

        let result = parse_date_to_millis("yesterday");
        assert!(result.is_err());

        let result = parse_date_to_millis("01-15-2024"); // wrong order
        assert!(result.is_err());
    }

    // ========================================================================
    // build_filter_params tests
    // ========================================================================

    #[test]
    fn test_build_filter_params_empty() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: None,
            email: None,
            since: None,
            until: None,
            sort_dir: SortDir::Desc,
            limit: None,
        };

        let result = build_filter_params(&args).unwrap();

        assert!(result.types.is_empty());
        assert!(result.org_types.is_empty());
        assert!(result.name.is_none());
        assert!(result.email.is_none());
        assert!(result.start.is_none());
        assert!(result.end.is_none());
        assert_eq!(result.sort_dir, Some("desc".to_string()));
        assert_eq!(result.page_size, Some(100)); // default
    }

    #[test]
    fn test_build_filter_params_activity_types() {
        let args = AuditFilterArgs {
            activity_type: vec!["SCAN_STARTED".to_string(), "SCAN_COMPLETED".to_string()],
            org_type: vec![],
            user: None,
            email: None,
            since: None,
            until: None,
            sort_dir: SortDir::Desc,
            limit: None,
        };

        let result = build_filter_params(&args).unwrap();
        assert_eq!(result.types.len(), 2);
        assert!(result.types.contains(&"SCAN_STARTED".to_string()));
        assert!(result.types.contains(&"SCAN_COMPLETED".to_string()));
    }

    #[test]
    fn test_build_filter_params_user_filters() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: Some("john".to_string()),
            email: Some("john@example.com".to_string()),
            since: None,
            until: None,
            sort_dir: SortDir::Desc,
            limit: None,
        };

        let result = build_filter_params(&args).unwrap();
        assert_eq!(result.name, Some("john".to_string()));
        assert_eq!(result.email, Some("john@example.com".to_string()));
    }

    #[test]
    fn test_build_filter_params_date_range() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: None,
            email: None,
            since: Some("2024-01-01".to_string()),
            until: Some("2024-01-31".to_string()),
            sort_dir: SortDir::Desc,
            limit: None,
        };

        let result = build_filter_params(&args).unwrap();
        assert!(result.start.is_some());
        assert!(result.end.is_some());
        // Verify start is before end
        assert!(result.start.unwrap() < result.end.unwrap());
    }

    #[test]
    fn test_build_filter_params_sort_asc() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: None,
            email: None,
            since: None,
            until: None,
            sort_dir: SortDir::Asc,
            limit: None,
        };

        let result = build_filter_params(&args).unwrap();
        assert_eq!(result.sort_dir, Some("asc".to_string()));
    }

    #[test]
    fn test_build_filter_params_limit() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: None,
            email: None,
            since: None,
            until: None,
            sort_dir: SortDir::Desc,
            limit: Some(50),
        };

        let result = build_filter_params(&args).unwrap();
        assert_eq!(result.page_size, Some(50));
    }

    #[test]
    fn test_build_filter_params_limit_caps_at_1000() {
        let args = AuditFilterArgs {
            activity_type: vec![],
            org_type: vec![],
            user: None,
            email: None,
            since: None,
            until: None,
            sort_dir: SortDir::Desc,
            limit: Some(5000), // over the max
        };

        let result = build_filter_params(&args).unwrap();
        assert_eq!(result.page_size, Some(1000)); // capped at 1000
    }
}
