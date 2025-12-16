//! Update configuration

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// Configuration for automatic updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update checking on startup
    #[serde(default = "default_true")]
    pub auto_check: bool,

    /// Automatically install updates (requires restart to take effect)
    #[serde(default)]
    pub auto_install: bool,

    /// Show update notifications on startup
    #[serde(default = "default_true")]
    pub notify: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_install: false,
            notify: true,
        }
    }
}
