//! Update manager for checking and installing updates

use self_update::cargo_crate_version;
use thiserror::Error;

/// Repository owner on GitHub
const REPO_OWNER: &str = "tarfu";
/// Repository name on GitHub
const REPO_NAME: &str = "tanuki-mcp";
/// Binary name
const BIN_NAME: &str = "tanuki-mcp";

/// Errors that can occur during update operations
#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Failed to check for updates: {0}")]
    CheckFailed(String),

    #[error("Failed to download update: {0}")]
    DownloadFailed(String),

    #[error("Failed to install update: {0}")]
    InstallFailed(String),

    #[error("Update cancelled by user")]
    Cancelled,
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
}

/// Manager for checking and performing updates
pub struct UpdateManager {
    repo_owner: String,
    repo_name: String,
    bin_name: String,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new() -> Self {
        Self {
            repo_owner: REPO_OWNER.to_string(),
            repo_name: REPO_NAME.to_string(),
            bin_name: BIN_NAME.to_string(),
        }
    }

    /// Get the current version
    pub fn current_version(&self) -> &'static str {
        cargo_crate_version!()
    }

    /// Check if an update is available
    ///
    /// Returns `Some(UpdateInfo)` if a newer version is available, `None` otherwise.
    pub fn check_for_updates(&self) -> Result<Option<UpdateInfo>, UpdateError> {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .build()
            .map_err(|e| UpdateError::CheckFailed(e.to_string()))?
            .fetch()
            .map_err(|e| UpdateError::CheckFailed(e.to_string()))?;

        let current = cargo_crate_version!();

        if let Some(latest) = releases.first() {
            let latest_version = &latest.version;

            if Self::is_newer_version(current, latest_version) {
                return Ok(Some(UpdateInfo {
                    current_version: current.to_string(),
                    latest_version: latest_version.clone(),
                }));
            }
        }

        Ok(None)
    }

    /// Perform the update interactively (with confirmation prompt)
    pub fn update(&self) -> Result<String, UpdateError> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .bin_name(&self.bin_name)
            .current_version(cargo_crate_version!())
            .show_download_progress(true)
            .show_output(true)
            .no_confirm(false)
            .build()
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?
            .update()
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?;

        Ok(status.version().to_string())
    }

    /// Perform the update without confirmation
    pub fn update_no_confirm(&self) -> Result<String, UpdateError> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .bin_name(&self.bin_name)
            .current_version(cargo_crate_version!())
            .show_download_progress(true)
            .show_output(false)
            .no_confirm(true)
            .build()
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?
            .update()
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?;

        Ok(status.version().to_string())
    }

    /// Check if the new version is actually newer using semver
    fn is_newer_version(current: &str, new: &str) -> bool {
        match (semver::Version::parse(current), semver::Version::parse(new)) {
            (Ok(curr_ver), Ok(new_ver)) => new_ver > curr_ver,
            _ => {
                // Fallback to string comparison if semver parsing fails
                new > current
            }
        }
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(UpdateManager::is_newer_version("0.1.0", "0.2.0"));
        assert!(!UpdateManager::is_newer_version("0.2.0", "0.1.0"));
        assert!(!UpdateManager::is_newer_version("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_semver_parsing() {
        assert!(UpdateManager::is_newer_version("0.1.1", "0.2.0"));
        assert!(UpdateManager::is_newer_version("0.1.1", "0.1.2"));
        assert!(UpdateManager::is_newer_version("0.1.1", "1.0.0"));
    }

    #[test]
    fn test_current_version() {
        let mgr = UpdateManager::new();
        assert!(!mgr.current_version().is_empty());
    }
}
