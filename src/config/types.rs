//! Configuration types for tanuki-mcp
//!
//! This module defines the configuration structure that can be loaded from
//! TOML files and/or environment variables.

use serde::Deserialize;
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// GitLab connection settings
    pub gitlab: GitLabConfig,

    /// Server/transport settings
    pub server: ServerConfig,

    /// Access control rules
    pub access_control: AccessControlConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Dashboard configuration
    pub dashboard: DashboardConfigToml,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gitlab: GitLabConfig::default(),
            server: ServerConfig::default(),
            access_control: AccessControlConfig::default(),
            logging: LoggingConfig::default(),
            dashboard: DashboardConfigToml::default(),
        }
    }
}

/// Dashboard configuration (TOML format)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DashboardConfigToml {
    /// Enable the dashboard server
    pub enabled: bool,

    /// Dashboard host
    pub host: String,

    /// Dashboard port
    pub port: u16,
}

impl Default for DashboardConfigToml {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 19892,
        }
    }
}

/// GitLab connection configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GitLabConfig {
    /// GitLab instance URL (e.g., `https://gitlab.com`)
    pub url: String,

    /// Personal Access Token (prefer env var GITLAB_TOKEN)
    #[serde(default)]
    pub token: Option<String>,

    /// API version (default: "v4")
    pub api_version: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum retries for failed requests
    pub max_retries: u32,

    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            url: "https://gitlab.com".to_string(),
            token: None,
            api_version: "v4".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            verify_ssl: true,
        }
    }
}

impl GitLabConfig {
    /// Get the full API base URL
    pub fn api_url(&self) -> String {
        format!(
            "{}/api/{}",
            self.url.trim_end_matches('/'),
            self.api_version
        )
    }
}

/// Server/transport configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Transport mode
    pub transport: TransportMode,

    /// HTTP host (for http/sse transport)
    pub host: String,

    /// HTTP port (for http/sse transport)
    pub port: u16,

    /// Server name for MCP
    pub name: String,

    /// Server version for MCP
    pub version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: TransportMode::Stdio,
            host: "127.0.0.1".to_string(),
            port: 20289,
            name: "tanuki-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Transport mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Standard input/output (default, for Claude Code)
    #[default]
    Stdio,
    /// HTTP with Server-Sent Events
    Http,
}

/// Access control configuration
///
/// The access control system uses a hierarchical override model:
/// 1. Global `all` level (base)
/// 2. Category-level overrides
/// 3. Individual action overrides
/// 4. Project-specific overrides (highest priority)
///
/// At each level, `deny` patterns are checked first, then `allow` patterns.
/// `allow` patterns can override `deny` patterns at the same level.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AccessControlConfig {
    /// Base access level for all tools
    pub all: AccessLevel,

    /// Global deny patterns (regex)
    #[serde(default)]
    pub deny: Vec<String>,

    /// Global allow patterns (regex, can override deny)
    #[serde(default)]
    pub allow: Vec<String>,

    /// Category-level access configuration
    #[serde(default)]
    pub categories: HashMap<String, CategoryAccessConfig>,

    /// Individual action overrides (highest priority after project)
    #[serde(default)]
    pub actions: HashMap<String, ActionPermission>,

    /// Per-project access overrides
    #[serde(default)]
    pub projects: HashMap<String, ProjectAccessConfig>,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            all: AccessLevel::Full,
            deny: Vec::new(),
            allow: Vec::new(),
            categories: HashMap::new(),
            actions: HashMap::new(),
            projects: HashMap::new(),
        }
    }
}

/// Base access level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// No access decision at this level (fall through to next level)
    #[default]
    None,
    /// Explicitly deny all operations
    Deny,
    /// Read-only operations (get, list, search)
    Read,
    /// Full access (all operations)
    Full,
}

/// Category-level access configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CategoryAccessConfig {
    /// Access level for this category
    pub level: AccessLevel,

    /// Deny patterns within this category
    #[serde(default)]
    pub deny: Vec<String>,

    /// Allow patterns within this category (can override deny)
    #[serde(default)]
    pub allow: Vec<String>,
}

impl Default for CategoryAccessConfig {
    fn default() -> Self {
        Self {
            level: AccessLevel::None,
            deny: Vec::new(),
            allow: Vec::new(),
        }
    }
}

/// Individual action permission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionPermission {
    /// Explicitly allow this action
    Allow,
    /// Explicitly deny this action
    Deny,
}

/// Project-specific access configuration
///
/// Inherits from global config but can override any setting
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProjectAccessConfig {
    /// Override base access level for this project
    #[serde(default)]
    pub all: Option<AccessLevel>,

    /// Additional deny patterns for this project
    #[serde(default)]
    pub deny: Vec<String>,

    /// Additional allow patterns for this project
    #[serde(default)]
    pub allow: Vec<String>,

    /// Category overrides for this project
    #[serde(default)]
    pub categories: HashMap<String, CategoryAccessConfig>,

    /// Action overrides for this project
    #[serde(default)]
    pub actions: HashMap<String, ActionPermission>,
}

impl Default for ProjectAccessConfig {
    fn default() -> Self {
        Self {
            all: None,
            deny: Vec::new(),
            allow: Vec::new(),
            categories: HashMap::new(),
            actions: HashMap::new(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Output format (pretty, json)
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
        }
    }
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable output
    #[default]
    Pretty,
    /// JSON structured output
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_config_api_url() {
        let config = GitLabConfig {
            url: "https://gitlab.example.com".to_string(),
            api_version: "v4".to_string(),
            ..Default::default()
        };
        assert_eq!(config.api_url(), "https://gitlab.example.com/api/v4");

        // Test with trailing slash
        let config = GitLabConfig {
            url: "https://gitlab.example.com/".to_string(),
            api_version: "v4".to_string(),
            ..Default::default()
        };
        assert_eq!(config.api_url(), "https://gitlab.example.com/api/v4");
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.gitlab.url, "https://gitlab.com");
        assert_eq!(config.gitlab.timeout_secs, 30);
        assert_eq!(config.server.transport, TransportMode::Stdio);
        assert_eq!(config.access_control.all, AccessLevel::Full);
    }

    #[test]
    fn test_deserialize_access_level() {
        let json = r#""read""#;
        let level: AccessLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, AccessLevel::Read);

        let json = r#""full""#;
        let level: AccessLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, AccessLevel::Full);

        let json = r#""none""#;
        let level: AccessLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, AccessLevel::None);

        let json = r#""deny""#;
        let level: AccessLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, AccessLevel::Deny);
    }

    #[test]
    fn test_deserialize_action_permission() {
        let json = r#""allow""#;
        let perm: ActionPermission = serde_json::from_str(json).unwrap();
        assert_eq!(perm, ActionPermission::Allow);

        let json = r#""deny""#;
        let perm: ActionPermission = serde_json::from_str(json).unwrap();
        assert_eq!(perm, ActionPermission::Deny);
    }
}
