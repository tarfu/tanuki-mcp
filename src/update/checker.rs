//! Background update checker
//!
//! Checks for updates on every startup if enabled.
//! Notifications are printed immediately from the background thread.

use super::{UpdateConfig, UpdateManager};

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

            if let Ok(Some(info)) = mgr.check_for_updates() {
                if auto_install {
                    if let Ok(version) = mgr.update_no_confirm()
                        && notify
                    {
                        eprintln!();
                        eprintln!("\x1b[32m  Update v{} installed!\x1b[0m", version);
                        eprintln!("\x1b[32m  Restart tanuki-mcp to use the new version\x1b[0m");
                        eprintln!();
                    }
                } else if notify {
                    // Just notify about available update (no auto-install)
                    eprintln!();
                    eprintln!(
                        "\x1b[33m  Update available: v{}\x1b[0m",
                        info.latest_version
                    );
                    eprintln!("\x1b[33m  Run: tanuki-mcp update\x1b[0m");
                    eprintln!();
                }
            }
        });
    }
}
