//! E2E tests for access control functionality.
//!
//! These tests verify that project-specific access control rules work correctly.
//! Unlike other tests that use the shared MCP server, these tests spawn dedicated
//! servers with custom access control configurations.

use crate::common;

use anyhow::{Context, Result};
use rmcp::model::{CallToolRequestParam, CallToolResult};
use rmcp::service::{Peer, RoleClient, ServiceExt};
use rmcp::transport::child_process::TokioChildProcess;
use serde_json::{Value, json};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::process::Command;

/// Helper to find the tanuki-mcp binary.
fn find_binary() -> Result<PathBuf> {
    for path in [
        "target/release/tanuki-mcp",
        "target/debug/tanuki-mcp",
        "../target/release/tanuki-mcp",
        "../target/debug/tanuki-mcp",
    ] {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!("tanuki-mcp binary not found")
}

/// Helper to get GitLab URL from environment.
fn get_gitlab_url() -> Result<String> {
    std::env::var("GITLAB_URL").context("GITLAB_URL not set")
}

/// Helper to get GitLab token from environment.
fn get_token() -> Result<String> {
    std::env::var("GITLAB_TOKEN").context("GITLAB_TOKEN not set")
}

/// A dedicated MCP client for access control testing.
struct AccessControlTestClient {
    peer: Peer<RoleClient>,
    _temp_dir: TempDir,
}

impl AccessControlTestClient {
    /// Create a new client with custom access control config.
    async fn new(config_toml: &str) -> Result<Self> {
        let binary_path = find_binary()?;
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_toml).context("Failed to write config")?;

        let mut cmd = Command::new(&binary_path);
        cmd.env("TANUKI_MCP_CONFIG", &config_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        let transport =
            TokioChildProcess::new(&mut cmd).context("Failed to create child process")?;

        let running_service =
            ().serve(transport)
                .await
                .context("Failed to start MCP client service")?;

        let peer = running_service.peer().clone();

        // Spawn the service in the background
        tokio::spawn(async move {
            let _ = running_service.waiting().await;
        });

        Ok(Self {
            peer,
            _temp_dir: temp_dir,
        })
    }

    /// Call a tool and return the result.
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult> {
        let args = match arguments {
            Value::Object(map) => Some(map),
            Value::Null => None,
            _ => anyhow::bail!("Arguments must be a JSON object or null"),
        };

        let param = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: args,
        };

        self.peer
            .call_tool(param)
            .await
            .context(format!("Failed to call tool: {}", name))
    }

    /// Call a tool and check if it was denied.
    async fn is_tool_denied(&self, name: &str, arguments: Value) -> bool {
        match self.call_tool(name, arguments).await {
            Ok(result) => result.is_error == Some(true),
            Err(_) => true,
        }
    }
}

/// Test that project-specific read-only access control works.
///
/// This test creates a server where a specific project pattern is restricted to read-only,
/// then verifies that:
/// 1. Read operations (list_branches) succeed
/// 2. Write operations (create_branch) are denied
#[tokio::test]
async fn test_project_read_only_access() {
    common::init_tracing();

    let gitlab_url = match get_gitlab_url() {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping test: GITLAB_URL not set");
            return;
        }
    };

    let token = match get_token() {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Skipping test: GITLAB_TOKEN not set");
            return;
        }
    };

    // Create a project for testing using the GitLab API directly
    let project_name = format!("restricted-{}", common::unique_name("ac"));
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{}/api/v4/projects", gitlab_url))
        .header("PRIVATE-TOKEN", &token)
        .json(&json!({
            "name": project_name,
            "visibility": "private",
            "initialize_with_readme": true
        }))
        .send()
        .await
        .expect("Failed to create project");

    if !create_resp.status().is_success() {
        panic!("Failed to create test project: {}", create_resp.status());
    }

    let project: Value = create_resp.json().await.expect("Failed to parse response");
    let project_path = project["path_with_namespace"]
        .as_str()
        .expect("No project path")
        .to_string();
    let project_id = project["id"].as_u64().expect("No project ID");

    // Create MCP client with project-specific read-only access
    let config = format!(
        r#"
[gitlab]
url = "{gitlab_url}"
token = "{token}"

[access_control]
all = "full"

[access_control.projects."{project_path}"]
all = "read"
"#,
        gitlab_url = gitlab_url,
        token = token,
        project_path = project_path
    );

    let mcp_client = AccessControlTestClient::new(&config)
        .await
        .expect("Failed to create MCP client");

    // Test 1: Read operation should succeed
    let list_result = mcp_client
        .call_tool("list_branches", json!({ "project": project_path }))
        .await
        .expect("list_branches should succeed");

    assert!(
        list_result.is_error != Some(true),
        "list_branches should not return error for read-only project"
    );

    // Test 2: Write operation should be denied
    let create_denied = mcp_client
        .is_tool_denied(
            "create_branch",
            json!({
                "project": project_path,
                "branch": "test-branch",
                "ref": "main"
            }),
        )
        .await;

    assert!(
        create_denied,
        "create_branch should be denied for read-only project"
    );

    // Cleanup: Delete the test project
    let _ = client
        .delete(format!("{}/api/v4/projects/{}", gitlab_url, project_id))
        .header("PRIVATE-TOKEN", &token)
        .send()
        .await;
}
