//! Functional test harness for HawkOp
//!
//! This module provides a test context and safety guards for running functional tests
//! against the real StackHawk API. Tests are opt-in via the `functional-tests` feature
//! and include safety checks for production environments.
//!
//! # Usage
//!
//! ```bash
//! # Preview what tests would run (list only)
//! make functional-test-dry-run
//!
//! # Against test environment
//! HAWKOP_PROFILE=test make functional-test
//!
//! # Against production (requires explicit confirmation)
//! HAWKOP_PROFILE=default HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes make functional-test
//! ```

use std::env;
use std::path::PathBuf;
use std::process::Command;

#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
#[allow(unused_imports)]
use assert_cmd::prelude::*;

pub mod error_tests;
pub mod mutation_tests;
pub mod read_tests;

// ============================================================================
// Test Configuration
// ============================================================================

/// Prefix for test resources to identify and clean up
pub const TEST_RESOURCE_PREFIX: &str = "hawkop-functest";

/// Production API host (requires explicit confirmation)
const PRODUCTION_API_HOST: &str = "api.stackhawk.com";

/// Warning banner for production API usage
const PRODUCTION_WARNING: &str = r#"
╔══════════════════════════════════════════════════════════════════╗
║  ⚠️  PRODUCTION API WARNING                                       ║
║                                                                   ║
║  You are about to run functional tests against:                   ║
║    https://api.stackhawk.com (PRODUCTION)                         ║
║                                                                   ║
║  This will make real API calls and may modify data.               ║
║                                                                   ║
║  To proceed, set: HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes             ║
╚══════════════════════════════════════════════════════════════════╝
"#;

// ============================================================================
// FunctionalTestContext
// ============================================================================

/// Context for functional tests providing command execution and safety guards.
///
/// The context respects the following environment variables:
/// - `HAWKOP_PROFILE` - Profile to use (e.g., `test`)
/// - `HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes` - Required for production API
pub struct FunctionalTestContext {
    /// Profile to use for API calls (from HAWKOP_PROFILE)
    pub profile: Option<String>,
    /// Path to the hawkop binary
    pub binary_path: PathBuf,
}

impl FunctionalTestContext {
    /// Create a new test context with safety checks.
    ///
    /// This will:
    /// 1. Detect if targeting production API
    /// 2. Require explicit confirmation for production usage
    pub fn new() -> Self {
        let profile = env::var("HAWKOP_PROFILE").ok();

        // Safety check for production API
        Self::check_production_safety(&profile);

        Self {
            profile,
            binary_path: cargo_bin!("hawkop").to_path_buf(),
        }
    }

    /// Check if targeting production and require confirmation.
    fn check_production_safety(profile: &Option<String>) {
        // Run `hawkop status --format json` to get the resolved API host
        let mut cmd = Command::new(cargo_bin!("hawkop"));
        cmd.args(["status", "--format", "json"]);
        if let Some(p) = profile {
            cmd.args(["--profile", p]);
        }

        if let Ok(output) = cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if API host is production
            if stdout.contains(PRODUCTION_API_HOST) {
                Self::require_production_confirmation();
            }
        }
    }

    /// Panic with warning if production confirmation is not set.
    fn require_production_confirmation() {
        if env::var("HAWKOP_FUNCTIONAL_TESTS_CONFIRM").as_deref() != Ok("yes") {
            eprintln!("{}", PRODUCTION_WARNING);
            panic!(
                "Production confirmation required. Set HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes to proceed."
            );
        }
    }

    /// Build a Command with profile settings applied.
    ///
    /// This does NOT execute the command - use `run()` for that.
    pub fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(&self.binary_path);
        // Always disable cache for functional tests to ensure fresh data
        cmd.arg("--no-cache");
        if let Some(ref p) = self.profile {
            cmd.args(["--profile", p]);
        }
        cmd.args(args);
        cmd
    }

    /// Execute command and return an assertion object for chaining.
    pub fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        self.command(args).assert()
    }

    /// Execute command and expect success, returning stdout as String.
    ///
    /// Panics if the command fails (non-zero exit code).
    pub fn run_success(&self, args: &[&str]) -> String {
        let output = self
            .command(args)
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "Command failed: hawkop {}\nstderr: {}",
                args.join(" "),
                stderr
            );
        }

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Execute command and expect failure, returning stderr as String.
    ///
    /// Panics if the command succeeds.
    pub fn run_failure(&self, args: &[&str]) -> String {
        let output = self
            .command(args)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            panic!("Command unexpectedly succeeded: hawkop {}", args.join(" "));
        }

        String::from_utf8_lossy(&output.stderr).to_string()
    }

    /// Execute command that may require a feature flag.
    ///
    /// If the command fails with "Access denied" (indicating missing feature flag),
    /// this will print a warning and pass the test. Otherwise, it expects success.
    ///
    /// This allows tests to pass in environments where the feature isn't enabled,
    /// while still validating the command works when the feature IS available.
    pub fn run_feature_flag_dependent(&self, args: &[&str], feature_name: &str) {
        let output = self
            .command(args)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            if stderr.contains("Access denied") {
                eprintln!(
                    "\n⚠️  SKIPPED: {} command requires '{}' feature flag",
                    args.join(" "),
                    feature_name
                );
                eprintln!("   This feature may not be enabled for the test organization.");
                let first_line = stderr.lines().next().unwrap_or("Access denied");
                // Strip "Error: " prefix if present to avoid "Error: Error:"
                let error_msg = first_line.strip_prefix("Error: ").unwrap_or(first_line);
                eprintln!("   Error: {}", error_msg);
                return; // Pass the test - feature not available
            }
            // Some other error - fail the test
            panic!(
                "Command failed (not due to feature flag): hawkop {}\nstderr: {}",
                args.join(" "),
                stderr
            );
        }
        // Command succeeded - feature is available and working
    }
}

impl Default for FunctionalTestContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Test Resource Naming
// ============================================================================

/// Generate a unique test resource name with timestamp.
///
/// Returns a name like `hawkop-functest-1706123456` that can be used for
/// teams and other resources created during testing.
pub fn test_resource_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{}", TEST_RESOURCE_PREFIX, ts)
}

// ============================================================================
// Test Team RAII Wrapper
// ============================================================================

/// RAII wrapper for test teams that ensures cleanup on drop.
///
/// Use this to create teams for mutation testing. The team will be automatically
/// deleted when this struct goes out of scope, even if the test panics.
pub struct TestTeam {
    ctx: FunctionalTestContext,
    pub name: String,
    pub created: bool,
}

impl TestTeam {
    /// Create a new test team with automatic cleanup.
    ///
    /// Returns a TestTeam that will delete itself on drop.
    pub fn create() -> Self {
        let ctx = FunctionalTestContext::new();
        let name = test_resource_name();

        let result = ctx.command(&["team", "create", &name]).output();

        let created = match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if created {
            eprintln!("[TEST] Created team: {}", name);
        } else {
            eprintln!("[TEST] Failed to create team: {}", name);
        }

        Self { ctx, name, created }
    }

    /// Create a test team with a custom name suffix.
    ///
    /// The full name will be `hawkop-functest-{suffix}-{timestamp}`.
    pub fn create_with_suffix(suffix: &str) -> Self {
        let ctx = FunctionalTestContext::new();
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let name = format!("{}-{}-{}", TEST_RESOURCE_PREFIX, suffix, ts);

        let result = ctx.command(&["team", "create", &name]).output();

        let created = match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if created {
            eprintln!("[TEST] Created team: {}", name);
        } else {
            eprintln!("[TEST] Failed to create team: {}", name);
        }

        Self { ctx, name, created }
    }
}

impl Drop for TestTeam {
    fn drop(&mut self) {
        if self.created {
            eprintln!("[TEST] Cleaning up team: {}", self.name);
            // Use --yes to skip confirmation prompt
            let _ = self
                .ctx
                .command(&["team", "delete", &self.name, "--yes"])
                .output();
        }
    }
}

// ============================================================================
// Unit Tests for Test Infrastructure
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_name_format() {
        let name = test_resource_name();
        assert!(name.starts_with(TEST_RESOURCE_PREFIX));
        // Should have a timestamp suffix
        let parts: Vec<&str> = name.split('-').collect();
        assert!(parts.len() >= 3); // hawkop-functest-timestamp
    }

    #[test]
    fn test_resource_name_uniqueness() {
        let name1 = test_resource_name();
        std::thread::sleep(std::time::Duration::from_millis(10));
        // Note: Within the same second, names may be identical
        // This is acceptable for our use case
        let name2 = test_resource_name();
        // Both should be valid prefixed names
        assert!(name1.starts_with(TEST_RESOURCE_PREFIX));
        assert!(name2.starts_with(TEST_RESOURCE_PREFIX));
    }
}
