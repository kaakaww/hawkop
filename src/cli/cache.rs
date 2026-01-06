//! Cache management commands

use crate::cache::CacheStorage;
use crate::cli::OutputFormat;
use crate::error::Result;

/// Show cache status/statistics
pub fn status(format: OutputFormat) -> Result<()> {
    let cache = CacheStorage::open().map_err(|e| crate::error::Error::Other(e.to_string()))?;
    let stats = cache
        .stats()
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "total_entries": stats.total_entries,
                "valid_entries": stats.valid_entries,
                "expired_entries": stats.expired_entries,
                "total_size_bytes": stats.total_size_bytes,
                "total_size_human": format_size(stats.total_size_bytes),
                "oldest_entry_timestamp": stats.oldest_entry,
                "newest_entry_timestamp": stats.newest_entry,
                "path": CacheStorage::cache_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            let path = CacheStorage::cache_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            println!("Cache Status");
            println!("────────────────────────────────────────");
            println!("Location:       {}", path);
            println!("Valid entries:  {}", stats.valid_entries);
            println!("Expired:        {}", stats.expired_entries);
            println!("Total size:     {}", format_size(stats.total_size_bytes));

            if let Some(oldest) = stats.oldest_entry {
                let dt = chrono::DateTime::from_timestamp(oldest, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                println!("Oldest entry:   {}", dt);
            }

            if let Some(newest) = stats.newest_entry {
                let dt = chrono::DateTime::from_timestamp(newest, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                println!("Newest entry:   {}", dt);
            }
        }
    }

    Ok(())
}

/// Clear all cache entries
pub fn clear(format: OutputFormat) -> Result<()> {
    let cache = CacheStorage::open().map_err(|e| crate::error::Error::Other(e.to_string()))?;
    let stats = cache
        .clear_all()
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "entries_removed": stats.entries_removed,
                "success": true,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            if stats.entries_removed > 0 {
                println!("Cleared {} cache entries", stats.entries_removed);
            } else {
                println!("Cache was already empty");
            }
        }
    }

    Ok(())
}

/// Show cache path
pub fn path() -> Result<()> {
    let path = CacheStorage::cache_dir().map_err(|e| crate::error::Error::Other(e.to_string()))?;
    println!("{}", path.display());
    Ok(())
}

/// Format bytes as human-readable size
fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
