//! Update checker for GitHub releases
//!
//! This module checks for new releases on GitHub and compares them to the current version.
//! It uses the GitHub API to fetch the latest release information.

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

/// GitHub repository information
const GITHUB_OWNER: &str = "evildarkarchon";
const GITHUB_REPO: &str = "ba2-batch-unpack-gui";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub API release response structure
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    /// Git tag name (e.g., "v1.0.0")
    tag_name: String,
    /// Human-readable release name
    name: String,
    /// Release notes/description (markdown)
    body: Option<String>,
    /// URL to the release page on GitHub
    html_url: String,
    /// Whether this is a pre-release version
    prerelease: bool,
    /// Whether this is a draft release
    draft: bool,
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current installed version
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Human-readable name of the latest release
    pub release_name: String,
    /// Release notes in markdown format
    pub release_notes: String,
    /// URL to download the update
    pub download_url: String,
    /// Whether the latest version is a pre-release
    pub is_prerelease: bool,
}

/// Check if an update is available
///
/// This function queries the GitHub API for the latest release and compares it
/// to the current version. It returns `Some(UpdateInfo)` if a newer version is available,
/// or `None` if the current version is up to date.
///
/// # Errors
///
/// Returns an error if:
/// - The GitHub API request fails
/// - The response cannot be parsed
/// - Version comparison fails
///
/// # Example
///
/// ```ignore
/// use unpackrr::update_checker::check_for_updates;
///
/// match check_for_updates().await {
///     Ok(Some(update)) => {
///         println!("Update available: {}", update.latest_version);
///     }
///     Ok(None) => {
///         println!("Already up to date!");
///     }
///     Err(e) => {
///         eprintln!("Failed to check for updates: {}", e);
///     }
/// }
/// ```
pub async fn check_for_updates() -> Result<Option<UpdateInfo>> {
    tracing::info!("Checking for updates from GitHub...");

    // Build GitHub API URL
    let url = format!(
        "https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest"
    );

    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .user_agent(format!("unpackrr/{CURRENT_VERSION}"))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch latest release from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub API returned error: {}",
            response.status()
        ));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Skip draft and prerelease versions (unless we want to include them)
    if release.draft {
        tracing::debug!("Latest release is a draft, skipping");
        return Ok(None);
    }

    // Parse version strings
    let current = parse_version(CURRENT_VERSION)?;
    let latest = parse_version(&release.tag_name)?;

    tracing::debug!(
        "Current version: {}, Latest version: {}",
        current,
        latest
    );

    // Compare versions
    if latest > current {
        tracing::info!("Update available: {} -> {}", current, latest);
        Ok(Some(UpdateInfo {
            current_version: current.to_string(),
            latest_version: latest.to_string(),
            release_name: release.name,
            release_notes: release.body.unwrap_or_default(),
            download_url: release.html_url,
            is_prerelease: release.prerelease,
        }))
    } else {
        tracing::info!("Already up to date ({})", current);
        Ok(None)
    }
}

/// Parse a version string, handling various formats
///
/// GitHub release tags often have a 'v' prefix (e.g., "v1.2.3"),
/// which semver doesn't accept. This function strips the prefix if present.
fn parse_version(version_str: &str) -> Result<Version> {
    let cleaned = version_str.trim_start_matches('v');
    Version::parse(cleaned).with_context(|| format!("Failed to parse version: {version_str}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_with_v_prefix() {
        let version = parse_version("v1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_parse_version_without_prefix() {
        let version = parse_version("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = parse_version("1.0.0").unwrap();
        let v2 = parse_version("1.1.0").unwrap();
        assert!(v2 > v1);
    }

    #[test]
    fn test_current_version_is_valid() {
        // Ensure CURRENT_VERSION can be parsed
        parse_version(CURRENT_VERSION).unwrap();
    }
}
