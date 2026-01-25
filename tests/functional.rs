//! Functional test entry point for HawkOp
//!
//! This file serves as the entry point for functional tests that exercise
//! HawkOp commands against the real StackHawk API.
//!
//! # Running Tests
//!
//! Functional tests are opt-in and require the `functional-tests` feature:
//!
//! ```bash
//! # Using Makefile (recommended)
//! HAWKOP_PROFILE=test make functional-test
//!
//! # Dry-run mode (no API calls)
//! HAWKOP_PROFILE=test make functional-test-dry-run
//!
//! # Using cargo directly
//! HAWKOP_PROFILE=test cargo test --features functional-tests --test functional
//! ```
//!
//! # Environment Variables
//!
//! - `HAWKOP_PROFILE` - Profile to use (e.g., `test`)
//! - `HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes` - Required for production API
//!
//! # Safety
//!
//! - Tests against `api.stackhawk.com` require explicit confirmation
//! - Mutation tests use `hawkop-functest-*` naming for easy identification
//! - Cleanup happens automatically via RAII pattern
//!
//! # Test Organization
//!
//! - `read_tests` - Safe read-only operations
//! - `mutation_tests` - Create/update/delete operations with cleanup
//! - `error_tests` - Expected failure scenarios

// Use path attribute to include modules from functional/ subdirectory
#[cfg(feature = "functional-tests")]
#[path = "functional/mod.rs"]
mod functional_harness;

// Re-export for test discovery
#[cfg(feature = "functional-tests")]
pub use functional_harness::*;
