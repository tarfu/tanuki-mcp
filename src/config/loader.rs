//! Configuration loader with layered sources
//!
//! Loads configuration from multiple sources with the following precedence
//! (highest to lowest):
//! 1. Environment variables (TANUKI_MCP_*)
//! 2. Configuration file (TOML)
//! 3. Default values

use crate::config::types::AppConfig;
use crate::error::ConfigError;
use config::{Config, Environment, File, FileFormat};
use std::path::Path;

/// Default configuration file paths to check (in order)
const DEFAULT_CONFIG_PATHS: &[&str] = &[
    "tanuki-mcp.toml",
    ".tanuki-mcp.toml",
    "~/.config/tanuki-mcp/config.toml",
    "/etc/tanuki-mcp/config.toml",
];

/// Load configuration from a TOML string (useful for testing)
pub fn load_config_from_str(toml_str: &str) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::from_str(toml_str, FileFormat::Toml))
        .build()
        .map_err(|e| ConfigError::Load(e.to_string()))?;

    let app_config: AppConfig = config
        .try_deserialize()
        .map_err(|e| ConfigError::Load(e.to_string()))?;

    // Skip token validation for testing
    validate_config_relaxed(&app_config)?;

    Ok(app_config)
}

/// Load configuration from files and environment
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, ConfigError> {
    let mut builder = Config::builder();

    // 1. Start with defaults (handled by serde defaults on AppConfig)

    // 2. Add configuration file
    if let Some(path) = config_path {
        // Explicit path provided - must exist
        if !Path::new(path).exists() {
            return Err(ConfigError::Load(format!(
                "Configuration file not found: {}",
                path
            )));
        }
        builder = builder.add_source(File::new(path, FileFormat::Toml));
    } else {
        // Try default paths (first existing one wins)
        for path in DEFAULT_CONFIG_PATHS {
            let expanded = shellexpand::tilde(path);
            if Path::new(expanded.as_ref()).exists() {
                builder = builder.add_source(File::new(&expanded, FileFormat::Toml));
                break;
            }
        }
    }

    // 3. Add environment variables with TANUKI_MCP_ prefix
    // e.g., TANUKI_MCP_GITLAB__URL, TANUKI_MCP_SERVER__PORT
    // Double underscore (__) maps to nested keys (gitlab.url)
    builder = builder.add_source(
        Environment::with_prefix("TANUKI_MCP")
            .separator("__")
            .try_parsing(true),
    );

    // 4. Handle common GitLab token environment variables
    // These are checked in order of precedence
    for env_var in &[
        "GITLAB_TOKEN",
        "GITLAB_PRIVATE_TOKEN",
        "GITLAB_ACCESS_TOKEN",
    ] {
        if let Ok(token) = std::env::var(env_var) {
            builder = builder
                .set_override("gitlab.token", token)
                .map_err(|e| ConfigError::Load(e.to_string()))?;
            break;
        }
    }

    // 5. Handle GITLAB_URL if set (common convention)
    if let Ok(url) = std::env::var("GITLAB_URL") {
        builder = builder
            .set_override("gitlab.url", url)
            .map_err(|e| ConfigError::Load(e.to_string()))?;
    }

    // Build and deserialize
    let config = builder
        .build()
        .map_err(|e| ConfigError::Load(e.to_string()))?;

    let app_config: AppConfig = config
        .try_deserialize()
        .map_err(|e| ConfigError::Load(e.to_string()))?;

    // Validate the configuration
    validate_config(&app_config)?;

    Ok(app_config)
}

/// Validate configuration values (relaxed - for testing without token)
fn validate_config_relaxed(config: &AppConfig) -> Result<(), ConfigError> {
    // Validate GitLab URL
    if config.gitlab.url.is_empty() {
        return Err(ConfigError::Missing {
            field: "gitlab.url".to_string(),
        });
    }

    if !config.gitlab.url.starts_with("http://") && !config.gitlab.url.starts_with("https://") {
        return Err(ConfigError::Invalid {
            message: format!(
                "gitlab.url must start with http:// or https://, got: {}",
                config.gitlab.url
            ),
        });
    }

    // Validate timeout
    if config.gitlab.timeout_secs == 0 {
        return Err(ConfigError::Invalid {
            message: "gitlab.timeout_secs must be greater than 0".to_string(),
        });
    }

    // Validate port
    if config.server.port == 0 {
        return Err(ConfigError::Invalid {
            message: "server.port must be greater than 0".to_string(),
        });
    }

    // Validate regex patterns
    validate_all_patterns(config)?;

    Ok(())
}

/// Validate all regex patterns in config
fn validate_all_patterns(config: &AppConfig) -> Result<(), ConfigError> {
    validate_patterns(&config.access_control.deny, "access_control.deny")?;
    validate_patterns(&config.access_control.allow, "access_control.allow")?;

    for (category, cat_config) in &config.access_control.categories {
        validate_patterns(
            &cat_config.deny,
            &format!("access_control.categories.{}.deny", category),
        )?;
        validate_patterns(
            &cat_config.allow,
            &format!("access_control.categories.{}.allow", category),
        )?;
    }

    for (project, proj_config) in &config.access_control.projects {
        validate_patterns(
            &proj_config.deny,
            &format!("access_control.projects.{}.deny", project),
        )?;
        validate_patterns(
            &proj_config.allow,
            &format!("access_control.projects.{}.allow", project),
        )?;

        for (category, cat_config) in &proj_config.categories {
            validate_patterns(
                &cat_config.deny,
                &format!(
                    "access_control.projects.{}.categories.{}.deny",
                    project, category
                ),
            )?;
            validate_patterns(
                &cat_config.allow,
                &format!(
                    "access_control.projects.{}.categories.{}.allow",
                    project, category
                ),
            )?;
        }
    }

    Ok(())
}

/// Validate configuration values
fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    // Validate GitLab URL
    if config.gitlab.url.is_empty() {
        return Err(ConfigError::Missing {
            field: "gitlab.url".to_string(),
        });
    }

    if !config.gitlab.url.starts_with("http://") && !config.gitlab.url.starts_with("https://") {
        return Err(ConfigError::Invalid {
            message: format!(
                "gitlab.url must start with http:// or https://, got: {}",
                config.gitlab.url
            ),
        });
    }

    // Token is required unless we add OAuth support later
    if config.gitlab.token.is_none() {
        return Err(ConfigError::Missing {
            field: "gitlab.token (set GITLAB_TOKEN environment variable)".to_string(),
        });
    }

    // Validate timeout
    if config.gitlab.timeout_secs == 0 {
        return Err(ConfigError::Invalid {
            message: "gitlab.timeout_secs must be greater than 0".to_string(),
        });
    }

    // Validate port
    if config.server.port == 0 {
        return Err(ConfigError::Invalid {
            message: "server.port must be greater than 0".to_string(),
        });
    }

    // Validate regex patterns in access control
    validate_all_patterns(config)?;

    Ok(())
}

/// Validate that all patterns are valid regex
fn validate_patterns(patterns: &[String], field_path: &str) -> Result<(), ConfigError> {
    for pattern in patterns {
        if let Err(e) = regex::Regex::new(pattern) {
            return Err(ConfigError::InvalidPattern {
                pattern: pattern.clone(),
                reason: format!("in {}: {}", field_path, e),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_from_str_basic() {
        let toml = r#"
[server]
name = "test-server"

[gitlab]
url = "https://gitlab.example.com"
token = "test-token"

[access_control]
all = "read"
"#;

        let config = load_config_from_str(toml).unwrap();
        assert_eq!(config.gitlab.url, "https://gitlab.example.com");
        assert_eq!(config.gitlab.token, Some("test-token".to_string()));
        assert_eq!(config.server.name, "test-server");
    }

    #[test]
    fn test_load_config_from_str_with_categories() {
        let toml = r#"
[server]
name = "test"

[gitlab]
url = "https://gitlab.com"
token = "token"

[access_control]
all = "read"

[access_control.categories.issues]
level = "full"
deny = ["delete_issue"]
"#;

        let config = load_config_from_str(toml).unwrap();
        let issues = config.access_control.categories.get("issues").unwrap();
        assert!(matches!(issues.level, crate::config::AccessLevel::Full));
        assert_eq!(issues.deny, vec!["delete_issue"]);
    }

    #[test]
    fn test_invalid_url_error() {
        let toml = r#"
[server]
name = "test"

[gitlab]
url = "not-a-url"
token = "token"

[access_control]
all = "read"
"#;

        let result = load_config_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let config = AppConfig {
            gitlab: crate::config::types::GitLabConfig {
                token: Some("test".to_string()),
                ..Default::default()
            },
            access_control: crate::config::types::AccessControlConfig {
                deny: vec!["[invalid".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidPattern { .. }
        ));
    }

    #[test]
    fn test_empty_url_error() {
        let toml = r#"
[server]
name = "test"

[gitlab]
url = ""
token = "token"

[access_control]
all = "read"
"#;

        let result = load_config_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_config() {
        let toml = r#"
[server]
name = "test"

[gitlab]
url = "https://gitlab.com"
token = "token"

[access_control]
all = "full"

[access_control.projects."prod/app"]
all = "read"
deny = [".*"]
allow = ["list_.*", "get_.*"]
"#;

        let config = load_config_from_str(toml).unwrap();
        let prod = config.access_control.projects.get("prod/app").unwrap();
        assert!(matches!(prod.all, Some(crate::config::AccessLevel::Read)));
        assert_eq!(prod.deny, vec![".*"]);
        assert_eq!(prod.allow, vec!["list_.*", "get_.*"]);
    }
}
