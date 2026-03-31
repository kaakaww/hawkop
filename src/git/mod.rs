//! Git repository detection for local working directory.
//!
//! Two-layer design:
//! - **Layer 1** (`detect_local_repo`): Pure git + string parsing, no API calls.
//!   Fast enough for inline nudges.
//! - **Layer 2** (`match_platform_repo`): Takes Layer 1 output + API client,
//!   matches against StackHawk's attack surface repos.

use log::debug;
use std::process::Command;

use crate::client::ListingApi;
use crate::client::models::Repository;
use crate::error::Result;

/// Git hosting provider, inferred from the remote URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitProvider {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    Unknown,
}

impl std::fmt::Display for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::GitLab => write!(f, "gitlab"),
            Self::Bitbucket => write!(f, "bitbucket"),
            Self::AzureDevOps => write!(f, "azure_devops"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about the local git repository, parsed from the remote URL.
#[derive(Debug, Clone)]
pub struct LocalRepoInfo {
    /// Raw remote URL as reported by git (retained for diagnostics and future use)
    #[allow(dead_code)]
    pub remote_url: String,
    /// Detected hosting provider
    pub provider: GitProvider,
    /// Organization or user (e.g., "kaakaww")
    pub owner: String,
    /// Repository name without .git suffix (e.g., "hawkop")
    pub name: String,
}

impl LocalRepoInfo {
    /// Returns "owner/name" display form (e.g., "kaakaww/hawkop").
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

// ─── Layer 1: Local detection (no API calls) ────────────────────────────────

/// Detect the git repository in the current working directory.
///
/// Runs `git remote get-url origin` and parses the result.
/// Returns `None` if not in a git repo, no remote named `origin`,
/// or the URL can't be parsed.
pub fn detect_local_repo() -> Option<LocalRepoInfo> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;

    if !output.status.success() {
        debug!("git remote get-url origin failed (not a git repo or no origin remote)");
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return None;
    }

    debug!("Detected git remote: {}", url);
    parse_remote_url(&url)
}

/// Parse a git remote URL into its components.
///
/// Handles:
/// - SSH: `git@github.com:owner/repo.git`
/// - HTTPS: `https://github.com/owner/repo.git`
/// - SSH with protocol: `ssh://git@github.com/owner/repo.git`
/// - Azure DevOps: `git@ssh.dev.azure.com:v3/org/project/repo`
/// - Azure DevOps HTTPS: `https://dev.azure.com/org/project/_git/repo`
pub fn parse_remote_url(url: &str) -> Option<LocalRepoInfo> {
    let url = url.trim();

    // Try SSH format: git@host:owner/repo.git
    if let Some(info) = parse_ssh_url(url) {
        return Some(info);
    }

    // Try HTTPS/SSH-protocol format: https://host/owner/repo.git
    if let Some(info) = parse_https_url(url) {
        return Some(info);
    }

    debug!("Could not parse git remote URL: {}", url);
    None
}

/// Parse SSH-style URLs: `git@github.com:owner/repo.git`
fn parse_ssh_url(url: &str) -> Option<LocalRepoInfo> {
    // Match: git@host:path or user@host:path
    let at_pos = url.find('@')?;
    let colon_pos = url[at_pos..].find(':').map(|p| p + at_pos)?;

    // Exclude ssh:// protocol URLs (they use / not :)
    if url.starts_with("ssh://") {
        return None;
    }

    let host = &url[at_pos + 1..colon_pos];
    let path = &url[colon_pos + 1..];

    // Azure DevOps SSH: git@ssh.dev.azure.com:v3/org/project/repo
    if host.contains("dev.azure.com") {
        return parse_azure_ssh_path(url, path);
    }

    let provider = provider_from_host(host);
    parse_owner_repo(url, path, provider)
}

/// Parse HTTPS-style URLs: `https://github.com/owner/repo.git`
/// Also handles `ssh://git@github.com/owner/repo.git`
fn parse_https_url(url: &str) -> Option<LocalRepoInfo> {
    // Strip protocol and optional user@ prefix to get "host/owner/repo.git"
    let after_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("ssh://"))?;

    // Strip optional user@ (e.g., "git@github.com/..." → "github.com/...")
    let stripped = match after_protocol.find('@') {
        Some(pos) => {
            let rest = &after_protocol[pos + 1..];
            // Only strip if @ comes before the first /
            if after_protocol[..pos].contains('/') {
                after_protocol
            } else {
                rest
            }
        }
        None => after_protocol,
    };

    // Split host from path: "github.com/owner/repo.git" → ("github.com", "owner/repo.git")
    let (host, path) = stripped.split_once('/')?;

    // Azure DevOps HTTPS: dev.azure.com/org/project/_git/repo
    if host.contains("dev.azure.com") {
        return parse_azure_https_path(url, path);
    }

    let provider = provider_from_host(host);
    parse_owner_repo(url, path, provider)
}

/// Parse "owner/repo.git" into LocalRepoInfo components.
fn parse_owner_repo(url: &str, path: &str, provider: GitProvider) -> Option<LocalRepoInfo> {
    let parts: Vec<&str> = path.splitn(3, '/').collect();
    if parts.len() < 2 {
        return None;
    }

    let owner = parts[0].to_string();
    let name = strip_git_suffix(parts[1]);

    if owner.is_empty() || name.is_empty() {
        return None;
    }

    Some(LocalRepoInfo {
        remote_url: url.to_string(),
        provider,
        owner,
        name,
    })
}

/// Parse Azure DevOps SSH path: `v3/org/project/repo`
fn parse_azure_ssh_path(url: &str, path: &str) -> Option<LocalRepoInfo> {
    let parts: Vec<&str> = path.split('/').collect();
    // v3/org/project/repo → parts[1]=org, parts[3]=repo
    if parts.len() >= 4 && parts[0] == "v3" {
        Some(LocalRepoInfo {
            remote_url: url.to_string(),
            provider: GitProvider::AzureDevOps,
            owner: parts[1].to_string(),
            name: strip_git_suffix(parts[3]),
        })
    } else {
        None
    }
}

/// Parse Azure DevOps HTTPS path: `org/project/_git/repo`
fn parse_azure_https_path(url: &str, path: &str) -> Option<LocalRepoInfo> {
    let parts: Vec<&str> = path.split('/').collect();
    // org/project/_git/repo → parts[0]=org, parts[3]=repo
    if parts.len() >= 4 && parts[2] == "_git" {
        Some(LocalRepoInfo {
            remote_url: url.to_string(),
            provider: GitProvider::AzureDevOps,
            owner: parts[0].to_string(),
            name: strip_git_suffix(parts[3]),
        })
    } else {
        None
    }
}

/// Infer provider from hostname.
fn provider_from_host(host: &str) -> GitProvider {
    let host_lower = host.to_lowercase();
    if host_lower.contains("github") {
        GitProvider::GitHub
    } else if host_lower.contains("gitlab") {
        GitProvider::GitLab
    } else if host_lower.contains("bitbucket") {
        GitProvider::Bitbucket
    } else if host_lower.contains("dev.azure.com") || host_lower.contains("visualstudio.com") {
        GitProvider::AzureDevOps
    } else {
        GitProvider::Unknown
    }
}

/// Strip trailing `.git` from a repo name.
fn strip_git_suffix(name: &str) -> String {
    name.strip_suffix(".git").unwrap_or(name).to_string()
}

// ─── Layer 2: Platform matching (requires API client) ────────────────────────

/// Match a local repo against StackHawk's attack surface repositories.
///
/// Tries matching by `owner/name` first (case-insensitive), then falls back
/// to name-only matching. Returns `None` if no match or ambiguous.
pub async fn match_platform_repo(
    client: &dyn ListingApi,
    org_id: &str,
    local: &LocalRepoInfo,
) -> Result<Option<Repository>> {
    let repos = client.list_repos(org_id, None).await?;

    debug!(
        "Matching local repo {}/{} against {} platform repos",
        local.owner,
        local.name,
        repos.len()
    );

    // First pass: match by owner/name (provider_org_name contains the owner)
    let full_matches: Vec<_> = repos
        .iter()
        .filter(|r| {
            let repo_name_matches = r.name.eq_ignore_ascii_case(&local.name);
            let owner_matches = r
                .provider_org_name
                .as_ref()
                .is_some_and(|org| org.eq_ignore_ascii_case(&local.owner));
            repo_name_matches && owner_matches
        })
        .collect();

    if full_matches.len() == 1 {
        debug!("Exact match: {}", full_matches[0].name);
        return Ok(Some(full_matches[0].clone()));
    }

    // Second pass: name-only fallback (for orgs where provider_org_name differs)
    let name_matches: Vec<_> = repos
        .iter()
        .filter(|r| r.name.eq_ignore_ascii_case(&local.name))
        .collect();

    if name_matches.len() == 1 {
        debug!(
            "Name-only match: {} (owner mismatch ignored)",
            name_matches[0].name
        );
        return Ok(Some(name_matches[0].clone()));
    }

    if name_matches.len() > 1 {
        debug!(
            "Ambiguous: {} repos match name \"{}\"",
            name_matches.len(),
            local.name
        );
    } else {
        debug!("No platform repo matches \"{}\"", local.full_name());
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════════
    // parse_remote_url tests
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn ssh_github() {
        let info = parse_remote_url("git@github.com:kaakaww/hawkop.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "kaakaww");
        assert_eq!(info.name, "hawkop");
        assert_eq!(info.full_name(), "kaakaww/hawkop");
    }

    #[test]
    fn ssh_github_no_dot_git() {
        let info = parse_remote_url("git@github.com:kaakaww/hawkop").unwrap();
        assert_eq!(info.name, "hawkop");
    }

    #[test]
    fn https_github() {
        let info = parse_remote_url("https://github.com/kaakaww/hawkop.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "kaakaww");
        assert_eq!(info.name, "hawkop");
    }

    #[test]
    fn https_github_no_dot_git() {
        let info = parse_remote_url("https://github.com/kaakaww/hawkop").unwrap();
        assert_eq!(info.name, "hawkop");
    }

    #[test]
    fn ssh_gitlab() {
        let info = parse_remote_url("git@gitlab.com:myorg/myrepo.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitLab);
        assert_eq!(info.owner, "myorg");
        assert_eq!(info.name, "myrepo");
    }

    #[test]
    fn https_gitlab() {
        let info = parse_remote_url("https://gitlab.com/myorg/myrepo.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitLab);
        assert_eq!(info.owner, "myorg");
        assert_eq!(info.name, "myrepo");
    }

    #[test]
    fn ssh_bitbucket() {
        let info = parse_remote_url("git@bitbucket.org:team/project.git").unwrap();
        assert_eq!(info.provider, GitProvider::Bitbucket);
        assert_eq!(info.owner, "team");
        assert_eq!(info.name, "project");
    }

    #[test]
    fn https_bitbucket() {
        let info = parse_remote_url("https://bitbucket.org/team/project.git").unwrap();
        assert_eq!(info.provider, GitProvider::Bitbucket);
        assert_eq!(info.owner, "team");
        assert_eq!(info.name, "project");
    }

    #[test]
    fn azure_devops_ssh() {
        let info = parse_remote_url("git@ssh.dev.azure.com:v3/myorg/myproject/myrepo").unwrap();
        assert_eq!(info.provider, GitProvider::AzureDevOps);
        assert_eq!(info.owner, "myorg");
        assert_eq!(info.name, "myrepo");
    }

    #[test]
    fn azure_devops_https() {
        let info = parse_remote_url("https://dev.azure.com/myorg/myproject/_git/myrepo").unwrap();
        assert_eq!(info.provider, GitProvider::AzureDevOps);
        assert_eq!(info.owner, "myorg");
        assert_eq!(info.name, "myrepo");
    }

    #[test]
    fn ssh_protocol_url() {
        let info = parse_remote_url("ssh://git@github.com/kaakaww/hawkop.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "kaakaww");
        assert_eq!(info.name, "hawkop");
    }

    #[test]
    fn unknown_host() {
        let info = parse_remote_url("git@selfhosted.example.com:team/project.git").unwrap();
        assert_eq!(info.provider, GitProvider::Unknown);
        assert_eq!(info.owner, "team");
        assert_eq!(info.name, "project");
    }

    #[test]
    fn http_url() {
        let info = parse_remote_url("http://github.com/kaakaww/hawkop.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "kaakaww");
        assert_eq!(info.name, "hawkop");
    }

    #[test]
    fn empty_url_returns_none() {
        assert!(parse_remote_url("").is_none());
    }

    #[test]
    fn garbage_url_returns_none() {
        assert!(parse_remote_url("not-a-url").is_none());
    }

    #[test]
    fn provider_display() {
        assert_eq!(GitProvider::GitHub.to_string(), "github");
        assert_eq!(GitProvider::GitLab.to_string(), "gitlab");
        assert_eq!(GitProvider::Bitbucket.to_string(), "bitbucket");
        assert_eq!(GitProvider::AzureDevOps.to_string(), "azure_devops");
        assert_eq!(GitProvider::Unknown.to_string(), "unknown");
    }
}
