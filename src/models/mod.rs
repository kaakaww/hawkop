//! Display models for CLI output
//!
//! This module provides shared display model abstractions for converting
//! API response types into CLI-friendly display formats.

pub mod display;

#[allow(unused_imports)] // AppDetailDisplay used in Sprint 3: app get/update
pub use display::{
    AlertDetail, AlertFindingDisplay, AlertMessageDetail, AppDetailDisplay, AppDisplay,
    AuditDisplay, ConfigDisplay, EnvDisplay, OASDisplay, OrgDisplay, PolicyDisplay,
    PrettyAlertDisplay, RepoDisplay, ScanDisplay, SecretDisplay, TeamListDisplay, UserDisplay,
};
