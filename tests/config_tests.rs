//! Configuration loading tests

use tanuki_mcp::config::{AccessLevel, TransportMode, load_config_from_str};

const MINIMAL_CONFIG: &str = r#"
[server]
name = "test-server"
version = "1.0.0"
transport = "stdio"

[gitlab]
url = "https://gitlab.example.com"
token = "test-token"

[access_control]
all = "read"
"#;

const FULL_CONFIG: &str = r#"
[server]
name = "tanuki-mcp-test"
version = "0.1.0"
transport = "http"
host = "0.0.0.0"
port = 9000

[gitlab]
url = "https://gitlab.company.com"
token = "glpat-test"
timeout_secs = 60
max_retries = 5
verify_ssl = false

[access_control]
all = "full"
deny = ["delete_.*"]
allow = ["delete_issue"]

[access_control.categories.issues]
level = "full"

[access_control.categories.merge_requests]
level = "full"
deny = ["merge_merge_request"]

[access_control.actions]
create_pipeline = "allow"
delete_project = "deny"

[access_control.projects."prod/app"]
all = "read"
deny = [".*"]
allow = ["list_.*", "get_.*"]
"#;

#[test]
fn test_minimal_config() {
    let config = load_config_from_str(MINIMAL_CONFIG).unwrap();

    assert_eq!(config.server.name, "test-server");
    assert_eq!(config.server.version, "1.0.0");
    assert!(matches!(config.server.transport, TransportMode::Stdio));

    assert_eq!(config.gitlab.url, "https://gitlab.example.com");
    assert_eq!(config.gitlab.token, Some("test-token".to_string()));

    assert!(matches!(config.access_control.all, AccessLevel::Read));
}

#[test]
fn test_full_config() {
    let config = load_config_from_str(FULL_CONFIG).unwrap();

    // Server
    assert_eq!(config.server.name, "tanuki-mcp-test");
    assert!(matches!(config.server.transport, TransportMode::Http));
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9000);

    // GitLab
    assert_eq!(config.gitlab.url, "https://gitlab.company.com");
    assert_eq!(config.gitlab.timeout_secs, 60);
    assert_eq!(config.gitlab.max_retries, 5);
    assert!(!config.gitlab.verify_ssl);

    // Access control base
    assert!(matches!(config.access_control.all, AccessLevel::Full));
    assert_eq!(config.access_control.deny, vec!["delete_.*"]);
    assert_eq!(config.access_control.allow, vec!["delete_issue"]);

    // Categories
    let issues = config.access_control.categories.get("issues").unwrap();
    assert!(matches!(issues.level, AccessLevel::Full));

    let mrs = config
        .access_control
        .categories
        .get("merge_requests")
        .unwrap();
    assert!(matches!(mrs.level, AccessLevel::Full));
    assert_eq!(mrs.deny, vec!["merge_merge_request"]);

    // Actions
    assert!(
        config
            .access_control
            .actions
            .contains_key("create_pipeline")
    );
    assert!(config.access_control.actions.contains_key("delete_project"));

    // Projects
    assert!(config.access_control.projects.contains_key("prod/app"));
    let prod = config.access_control.projects.get("prod/app").unwrap();
    assert!(matches!(prod.all, Some(AccessLevel::Read)));
}

#[test]
fn test_config_defaults() {
    let config_str = r#"
[server]
name = "test"

[gitlab]
url = "https://gitlab.com"
token = "token"

[access_control]
all = "read"
"#;

    let config = load_config_from_str(config_str).unwrap();

    // Check defaults
    assert!(matches!(config.server.transport, TransportMode::Stdio)); // Default transport
    assert_eq!(config.server.host, "127.0.0.1"); // Default host
    assert_eq!(config.server.port, 20289); // Default port
    assert_eq!(config.gitlab.timeout_secs, 30); // Default timeout
    assert_eq!(config.gitlab.max_retries, 3); // Default retries
    assert!(config.gitlab.verify_ssl); // Default verify_ssl
}

#[test]
fn test_config_uses_default_gitlab_url() {
    let config_str = r#"
[server]
name = "test"

[gitlab]
token = "token"

[access_control]
all = "read"
"#;

    // Missing url should use default "https://gitlab.com"
    let config = load_config_from_str(config_str).unwrap();
    assert_eq!(config.gitlab.url, "https://gitlab.com");
}

#[test]
fn test_invalid_regex_pattern() {
    let config_str = r#"
[server]
name = "test"

[gitlab]
url = "https://gitlab.com"
token = "token"

[access_control]
all = "read"
deny = ["[invalid"]
"#;

    let result = load_config_from_str(config_str);
    assert!(result.is_err());
}

#[test]
#[serial_test::serial]
fn test_env_var_priority_tanuki_mcp_over_gitlab_token() {
    use std::env;
    use std::fs;
    use tanuki_mcp::config::load_config;
    use tempfile::tempdir;

    // Create a temporary config file without a token
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test-config.toml");
    let config_content = r#"
[gitlab]
url = "https://gitlab.com"

[access_control]
all = "read"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Set both TANUKI_MCP__GITLAB_TOKEN and GITLAB_TOKEN
    unsafe {
        env::set_var("TANUKI_MCP__GITLAB_TOKEN", "tanuki-priority-token");
        env::set_var("GITLAB_TOKEN", "gitlab-fallback-token");
    }

    // Load config
    let config = load_config(Some(config_path.to_str().unwrap())).unwrap();

    // TANUKI_MCP__GITLAB_TOKEN should take precedence
    assert_eq!(
        config.gitlab.token,
        Some("tanuki-priority-token".to_string())
    );

    // Cleanup
    unsafe {
        env::remove_var("TANUKI_MCP__GITLAB_TOKEN");
        env::remove_var("GITLAB_TOKEN");
    }
}

#[test]
#[serial_test::serial]
fn test_env_var_gitlab_token_fallback() {
    use std::env;
    use std::fs;
    use tanuki_mcp::config::load_config;
    use tempfile::tempdir;

    // Create a temporary config file without a token
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test-config.toml");
    let config_content = r#"
[gitlab]
url = "https://gitlab.com"

[access_control]
all = "read"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Ensure TANUKI_MCP__GITLAB_TOKEN is not set, only GITLAB_TOKEN
    unsafe {
        env::remove_var("TANUKI_MCP__GITLAB_TOKEN");
        env::set_var("GITLAB_TOKEN", "gitlab-fallback-token");
    }

    // Load config
    let config = load_config(Some(config_path.to_str().unwrap())).unwrap();

    // GITLAB_TOKEN should be used as fallback
    assert_eq!(
        config.gitlab.token,
        Some("gitlab-fallback-token".to_string())
    );

    // Cleanup
    unsafe {
        env::remove_var("GITLAB_TOKEN");
    }
}

#[test]
#[serial_test::serial]
fn test_env_var_priority_tanuki_mcp_over_gitlab_url() {
    use std::env;
    use std::fs;
    use tanuki_mcp::config::load_config;
    use tempfile::tempdir;

    // Create a temporary config file with minimal settings
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test-config.toml");
    let config_content = r#"
[gitlab]
token = "test-token"

[access_control]
all = "read"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Set both TANUKI_MCP__GITLAB_URL and GITLAB_URL
    unsafe {
        env::set_var(
            "TANUKI_MCP__GITLAB_URL",
            "https://tanuki-priority.gitlab.com",
        );
        env::set_var("GITLAB_URL", "https://fallback.gitlab.com");
    }

    // Load config
    let config = load_config(Some(config_path.to_str().unwrap())).unwrap();

    // TANUKI_MCP__GITLAB_URL should take precedence
    assert_eq!(config.gitlab.url, "https://tanuki-priority.gitlab.com");

    // Cleanup
    unsafe {
        env::remove_var("TANUKI_MCP__GITLAB_URL");
        env::remove_var("GITLAB_URL");
    }
}

#[test]
#[serial_test::serial]
fn test_env_var_gitlab_url_fallback() {
    use std::env;
    use std::fs;
    use tanuki_mcp::config::load_config;
    use tempfile::tempdir;

    // Create a temporary config file with minimal settings
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test-config.toml");
    let config_content = r#"
[gitlab]
token = "test-token"

[access_control]
all = "read"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Ensure TANUKI_MCP__GITLAB_URL is not set, only GITLAB_URL
    unsafe {
        env::remove_var("TANUKI_MCP__GITLAB_URL");
        env::set_var("GITLAB_URL", "https://fallback.gitlab.com");
    }

    // Load config
    let config = load_config(Some(config_path.to_str().unwrap())).unwrap();

    // GITLAB_URL should be used as fallback
    assert_eq!(config.gitlab.url, "https://fallback.gitlab.com");

    // Cleanup
    unsafe {
        env::remove_var("GITLAB_URL");
    }
}
