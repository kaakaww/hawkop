//! Display models for run command output
//!
//! These models format Perch device status for CLI display.

use colored::Colorize;
use serde::Serialize;
use tabled::Tabled;

use crate::client::models::PerchDevice;

/// Display model for scan runner status
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct RunStatusDisplay {
    #[tabled(rename = "STATUS")]
    pub status: String,

    #[tabled(rename = "APP ID")]
    pub app_id: String,

    #[tabled(rename = "RUNNER")]
    pub runner_name: String,

    #[tabled(rename = "COMMAND")]
    pub current_command: String,

    #[tabled(rename = "STARTED")]
    pub started_at: String,
}

impl From<PerchDevice> for RunStatusDisplay {
    fn from(device: PerchDevice) -> Self {
        let status_str = device
            .status
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        // Color the status based on state
        let status = match status_str.to_uppercase().as_str() {
            "RUNNING" | "SCANNING" => status_str.green().to_string(),
            "IDLE" | "STOPPED" | "COMPLETE" => status_str.blue().to_string(),
            "ERROR" | "FAILED" => status_str.red().to_string(),
            "NO_DEVICE" => "No hosted runner".yellow().to_string(),
            _ => status_str,
        };

        let current_command = device
            .command
            .as_ref()
            .and_then(|c| c.command.clone())
            .unwrap_or_else(|| "-".to_string());

        let started_at = device
            .created_date
            .map(|ts| {
                chrono::DateTime::from_timestamp(ts / 1000, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| ts.to_string())
            })
            .unwrap_or_else(|| "-".to_string());

        Self {
            status,
            app_id: device.application_id.unwrap_or_else(|| "-".to_string()),
            runner_name: device.name.unwrap_or_else(|| "-".to_string()),
            current_command,
            started_at,
        }
    }
}

/// Pretty print format for scan status (used in non-table output)
pub struct PrettyRunStatus<'a> {
    pub device: &'a PerchDevice,
    pub app_name: Option<&'a str>,
}

impl<'a> PrettyRunStatus<'a> {
    pub fn new(device: &'a PerchDevice, app_name: Option<&'a str>) -> Self {
        Self { device, app_name }
    }

    /// Print a formatted status report
    pub fn print(&self) {
        let status = self
            .device
            .status
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let is_running = self.device.is_running();

        // Header with app name if available
        if let Some(name) = self.app_name {
            println!("{} {}", "Application:".bold(), name);
        }
        if let Some(app_id) = &self.device.application_id {
            println!("{} {}", "App ID:".bold(), app_id);
        }
        println!();

        // Status line with color
        let status_display = match status.to_uppercase().as_str() {
            "RUNNING" | "SCANNING" => format!("● {}", status).green(),
            "IDLE" | "STOPPED" | "COMPLETE" => format!("○ {}", status).blue(),
            "ERROR" | "FAILED" => format!("✗ {}", status).red(),
            "NO_DEVICE" => "○ No hosted runner configured".yellow(),
            _ => format!("? {}", status).normal(),
        };
        println!("{} {}", "Status:".bold(), status_display);

        // Runner details
        if let Some(name) = &self.device.name {
            println!("{} {}", "Runner:".bold(), name);
        }

        // Current command
        if let Some(cmd) = &self.device.command {
            if let Some(command) = &cmd.command {
                println!("{} {}", "Command:".bold(), command);
            }
            if let Some(url) = &cmd.target_url {
                println!("{} {}", "Target:".bold(), url);
            }
            if let Some(error) = &cmd.error
                && let Some(msg) = &error.error_message
            {
                println!("{} {}", "Error:".bold().red(), msg.red());
            }
        }

        // Timestamps
        if let Some(ts) = self.device.created_date
            && let Some(dt) = chrono::DateTime::from_timestamp(ts / 1000, 0)
        {
            println!(
                "{} {}",
                "Started:".bold(),
                dt.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }

        // Suggestion for next steps
        println!();
        if is_running {
            println!(
                "{}",
                "→ Use `hawkop run stop --app <app>` to stop the scan".dimmed()
            );
        } else if status.to_uppercase() == "NO_DEVICE" {
            println!(
                "{}",
                "→ Use `hawkop run start --app <app>` to start a scan".dimmed()
            );
        } else {
            println!(
                "{}",
                "→ Use `hawkop run start --app <app>` to start a new scan".dimmed()
            );
            println!(
                "{}",
                "→ Use `hawkop scan list --app <app>` to view scan results".dimmed()
            );
        }
    }
}
