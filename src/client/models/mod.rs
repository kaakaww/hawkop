//! StackHawk API data models
//!
//! This module contains all the domain types returned by the StackHawk API.
//! Models are organized by resource type for easy discovery.

// Allow unused imports - we export all API types for completeness,
// even if not all are currently used by CLI commands.
#![allow(unused_imports)]

mod app;
mod audit;
mod auth;
mod config;
mod finding;
mod oas;
mod org;
mod policy;
mod repo;
mod scan;
mod secret;
mod user;

// Re-export all models for convenient access
pub use app::{Application, CloudScanTarget};
pub use audit::{AuditFilterParams, AuditRecord};
pub use auth::JwtToken;
pub use config::ScanConfig;
pub use finding::{
    AlertMsgResponse, AlertResponse, ApplicationAlert, ApplicationAlertUri, ScanAlertsResponse,
    ScanMessage, ScanResultWithAlerts,
};
pub use oas::OASAsset;
pub use org::Organization;
pub use policy::{OrgPolicy, PolicyType, StackHawkPolicy};
pub use repo::{
    OpenApiSpecInfo, RepoAppInfo, RepoContributor, RepoInsight, Repository, SensitiveDataTag,
};
pub use scan::{AlertStats, AlertStatusStats, Scan, ScanMetadata, ScanResult, ScanTag};
pub use secret::Secret;
pub use user::{
    CreateTeamRequest, Team, TeamApplication, TeamDetail, TeamUser, UpdateApplicationTeamRequest,
    UpdateTeamRequest, User, UserExternal,
};
