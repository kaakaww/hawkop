//! Repository display model

use serde::Serialize;
use tabled::Tabled;

use super::common::format_as_iso_datetime;
use crate::client::models::Repository;

/// Repository display model for table/JSON output.
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct RepoDisplay {
    /// Whether repo is in attack surface
    #[tabled(rename = "SURFACE")]
    pub attack_surface: String,

    /// Git provider (GITHUB, AZURE_DEVOPS, BITBUCKET, GITLAB)
    #[tabled(rename = "SRC")]
    pub provider: String,

    /// Git organization name
    #[tabled(rename = "ORG")]
    pub git_org: String,

    /// Repository name
    #[tabled(rename = "REPO")]
    pub name: String,

    /// Whether StackHawk generated an OAS
    #[tabled(rename = "OAS")]
    pub oas: String,

    /// Sensitive data tags
    #[tabled(rename = "SENSITIVE")]
    pub sensitive_data: String,

    /// Last commit time (ISO datetime)
    #[tabled(rename = "COMMITTED")]
    pub last_commit: String,

    /// Last committer
    #[tabled(rename = "BY")]
    pub last_committer: String,

    /// 30-day commit activity
    #[tabled(rename = "30D")]
    pub commit_count: String,

    /// Number of mapped apps
    #[tabled(rename = "APPS")]
    pub app_count: String,
}

impl From<Repository> for RepoDisplay {
    fn from(repo: Repository) -> Self {
        // Format provider
        let provider = repo.repo_source.unwrap_or_else(|| "--".to_string());

        // Format git org
        let git_org = repo.provider_org_name.unwrap_or_else(|| "--".to_string());

        // Format attack surface boolean (checkmark or empty)
        let attack_surface = if repo.is_in_attack_surface {
            "\u{2713}".to_string() // checkmark
        } else {
            "".to_string()
        };

        // Format OAS boolean (checkmark or empty)
        let oas = if repo.has_generated_open_api_spec {
            "\u{2713}".to_string() // checkmark
        } else {
            "".to_string()
        };

        // Format sensitive data tags (comma-separated, truncated)
        let sensitive_data = if repo.sensitive_data_tags.is_empty() {
            "--".to_string()
        } else {
            let tags: Vec<String> = repo
                .sensitive_data_tags
                .iter()
                .map(|t| t.name.clone())
                .collect();
            let joined = tags.join(", ");
            if joined.len() > 20 {
                format!("{}...", &joined[..17])
            } else {
                joined
            }
        };

        // Format last commit time as ISO datetime
        let last_commit = repo
            .last_commit_timestamp
            .as_ref()
            .map(|ts| format_as_iso_datetime(ts))
            .unwrap_or_else(|| "--".to_string());

        // Format last committer
        let last_committer = repo
            .last_contributor
            .and_then(|c| c.name)
            .unwrap_or_else(|| "--".to_string());

        // Format commit count
        let commit_count = repo.commit_count.to_string();

        // Format app count
        let app_count = repo.app_infos.len().to_string();

        Self {
            attack_surface,
            provider,
            git_org,
            name: repo.name,
            oas,
            sensitive_data,
            last_commit,
            last_committer,
            commit_count,
            app_count,
        }
    }
}

impl From<&Repository> for RepoDisplay {
    fn from(repo: &Repository) -> Self {
        RepoDisplay::from(repo.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_display_from_repository() {
        let repo = Repository {
            id: Some("repo-123".to_string()),
            repo_source: Some("GITHUB".to_string()),
            provider_org_name: Some("myorg".to_string()),
            name: "my-api".to_string(),
            open_api_spec_info: None,
            has_generated_open_api_spec: true,
            is_in_attack_surface: true,
            framework_names: vec!["Spring".to_string()],
            sensitive_data_tags: vec![],
            last_commit_timestamp: None,
            last_contributor: None,
            commit_count: 10,
            app_infos: vec![],
            insights: vec![],
        };

        let display = RepoDisplay::from(repo);

        assert_eq!(display.name, "my-api");
        assert_eq!(display.provider, "GITHUB");
        assert_eq!(display.attack_surface, "\u{2713}"); // checkmark
        assert_eq!(display.oas, "\u{2713}"); // checkmark
    }

    #[test]
    fn test_repo_display_not_in_attack_surface() {
        let repo = Repository {
            id: Some("repo-456".to_string()),
            repo_source: Some("GITLAB".to_string()),
            provider_org_name: None,
            name: "internal-tool".to_string(),
            open_api_spec_info: None,
            has_generated_open_api_spec: false,
            is_in_attack_surface: false,
            framework_names: vec![],
            sensitive_data_tags: vec![],
            last_commit_timestamp: None,
            last_contributor: None,
            commit_count: 0,
            app_infos: vec![],
            insights: vec![],
        };

        let display = RepoDisplay::from(repo);

        assert_eq!(display.name, "internal-tool");
        assert_eq!(display.attack_surface, ""); // Empty for false
        assert_eq!(display.oas, ""); // Empty for false
    }
}
