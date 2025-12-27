//! Audit log management commands

use chrono::{Duration, Utc};

use crate::cli::{AuditFilterArgs, CommandContext, OutputFormat, SortDir};
use crate::client::{AuditFilterParams, StackHawkApi};
use crate::error::Result;
use crate::models::AuditDisplay;
use crate::output::Formattable;

/// Run the audit list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    filters: &AuditFilterArgs,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;

    // Convert CLI args to API filter params
    let api_filters = build_filter_params(filters)?;

    let org_id = ctx.require_org_id()?;
    let records = ctx
        .client
        .list_audit(org_id, Some(&api_filters))
        .await?;

    let display_records: Vec<AuditDisplay> =
        records.into_iter().map(AuditDisplay::from).collect();
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
    if let Some(stripped) = date_str.strip_suffix('d') {
        if let Ok(days) = stripped.parse::<i64>() {
            let target = now - Duration::days(days);
            return Ok(target.timestamp_millis());
        }
    }

    if let Some(stripped) = date_str.strip_suffix('w') {
        if let Ok(weeks) = stripped.parse::<i64>() {
            let target = now - Duration::weeks(weeks);
            return Ok(target.timestamp_millis());
        }
    }

    if let Some(stripped) = date_str.strip_suffix('h') {
        if let Ok(hours) = stripped.parse::<i64>() {
            let target = now - Duration::hours(hours);
            return Ok(target.timestamp_millis());
        }
    }

    // Try ISO date format (YYYY-MM-DD)
    if date_str.len() == 10 && date_str.chars().nth(4) == Some('-') {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let datetime = date
                .and_hms_opt(0, 0, 0)
                .expect("valid time")
                .and_utc();
            return Ok(datetime.timestamp_millis());
        }
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
