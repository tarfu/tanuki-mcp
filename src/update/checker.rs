//! Background update checker
//!
//! Checks for updates on every startup if enabled.
//! Notifications are printed immediately from the background thread.

use super::{UpdateConfig, UpdateManager};

/// Action to take after checking for updates
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateAction {
    /// No action needed
    None,
    /// Notify user about available update
    NotifyAvailable { version: String },
    /// Notify user that update was installed
    NotifyInstalled { version: String },
}

/// Determine what update action to take based on check results and config.
///
/// This is a pure function that encapsulates the update decision logic,
/// making it easy to test without side effects.
pub fn determine_update_action(
    available_version: Option<&str>,
    auto_install: bool,
    notify: bool,
    installed_version: Option<&str>,
) -> UpdateAction {
    // No update available - nothing to do
    let Some(version) = available_version else {
        return UpdateAction::None;
    };

    // Auto-install was enabled and succeeded
    if auto_install {
        if let Some(installed) = installed_version
            && notify
        {
            return UpdateAction::NotifyInstalled {
                version: installed.to_string(),
            };
        }
        // Auto-install failed or notify disabled - nothing to show
        return UpdateAction::None;
    }

    // No auto-install, just notify about available update
    if notify {
        return UpdateAction::NotifyAvailable {
            version: version.to_string(),
        };
    }

    UpdateAction::None
}

/// Background update checker
pub struct UpdateChecker {
    enabled: bool,
    auto_install: bool,
    notify: bool,
}

impl UpdateChecker {
    /// Create a new update checker from config
    pub fn new(config: &UpdateConfig) -> Self {
        Self {
            enabled: config.auto_check,
            auto_install: config.auto_install,
            notify: config.notify,
        }
    }

    /// Perform a background update check (non-blocking)
    ///
    /// Checks on every startup if enabled. If `auto_install` is enabled,
    /// will automatically install the update and print a notification.
    /// If `auto_install` is disabled but `notify` is enabled, will print
    /// a notification about the available update.
    pub fn check_in_background(&self) {
        if !self.enabled {
            return;
        }

        let auto_install = self.auto_install;
        let notify = self.notify;

        // Spawn a background thread for the update check
        std::thread::spawn(move || {
            let mgr = UpdateManager::new();

            let available_version = mgr
                .check_for_updates()
                .ok()
                .flatten()
                .map(|info| info.latest_version);

            let installed_version = if available_version.is_some() && auto_install {
                mgr.update_no_confirm().ok()
            } else {
                None
            };

            let action = determine_update_action(
                available_version.as_deref(),
                auto_install,
                notify,
                installed_version.as_deref(),
            );

            match action {
                UpdateAction::None => {}
                UpdateAction::NotifyAvailable { version } => {
                    eprintln!();
                    eprintln!("\x1b[33m  Update available: v{}\x1b[0m", version);
                    eprintln!("\x1b[33m  Run: tanuki-mcp update\x1b[0m");
                    eprintln!();
                }
                UpdateAction::NotifyInstalled { version } => {
                    eprintln!();
                    eprintln!("\x1b[32m  Update v{} installed!\x1b[0m", version);
                    eprintln!("\x1b[32m  Restart tanuki-mcp to use the new version\x1b[0m");
                    eprintln!();
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_update_available() {
        let action = determine_update_action(None, false, true, None);
        assert_eq!(action, UpdateAction::None);
    }

    #[test]
    fn test_update_available_notify_enabled() {
        let action = determine_update_action(Some("1.0.0"), false, true, None);
        assert_eq!(
            action,
            UpdateAction::NotifyAvailable {
                version: "1.0.0".to_string()
            }
        );
    }

    #[test]
    fn test_update_available_notify_disabled() {
        let action = determine_update_action(Some("1.0.0"), false, false, None);
        assert_eq!(action, UpdateAction::None);
    }

    #[test]
    fn test_auto_install_succeeded_notify_enabled() {
        let action = determine_update_action(Some("1.0.0"), true, true, Some("1.0.0"));
        assert_eq!(
            action,
            UpdateAction::NotifyInstalled {
                version: "1.0.0".to_string()
            }
        );
    }

    #[test]
    fn test_auto_install_succeeded_notify_disabled() {
        let action = determine_update_action(Some("1.0.0"), true, false, Some("1.0.0"));
        assert_eq!(action, UpdateAction::None);
    }

    #[test]
    fn test_auto_install_failed() {
        // auto_install enabled but installed_version is None (install failed)
        let action = determine_update_action(Some("1.0.0"), true, true, None);
        assert_eq!(action, UpdateAction::None);
    }

    #[test]
    fn test_all_disabled() {
        let action = determine_update_action(Some("1.0.0"), false, false, None);
        assert_eq!(action, UpdateAction::None);
    }
}
