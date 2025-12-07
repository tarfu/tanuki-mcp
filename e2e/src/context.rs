//! Test context for E2E tests.
//!
//! Provides a unified context that combines:
//! - MCP client (via shared servers for stdio or HTTP transport)
//! - GitLab container access
//! - Test project management

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::gitlab::GitLabContainer;
use crate::shared::{SharedMcpClient, get_shared_servers};
use crate::transport::TransportKind;

/// Test context for E2E tests.
///
/// Provides access to:
/// - An MCP client connected via the specified transport (using shared server)
/// - GitLab container for direct API access
/// - Test project information
pub struct TestContext {
    /// The MCP client (backed by shared server).
    pub client: SharedMcpClient,
    /// The GitLab container manager (borrowed from shared servers).
    pub gitlab: &'static GitLabContainer,
    /// The Personal Access Token for GitLab.
    pub token: String,
    /// The test project ID (if created).
    pub project_id: Option<u64>,
    /// The test project path (if created).
    pub project_path: Option<String>,
}

impl TestContext {
    /// Create a new test context with the specified transport.
    ///
    /// This uses the shared MCP server for the specified transport,
    /// avoiding the overhead of spawning a new server per test.
    ///
    /// Returns `None` if the requested transport is not available
    /// (e.g., HTTP transport when `MCP_HTTP_URL` is not set).
    pub async fn new(transport: TransportKind) -> Result<Option<Self>> {
        let servers = get_shared_servers().await;

        let peer = match servers.get_peer(transport) {
            Some(peer) => peer,
            None => {
                tracing::info!("Transport {:?} not available, skipping test", transport);
                return Ok(None);
            }
        };

        Ok(Some(Self {
            client: SharedMcpClient::new(peer, transport),
            gitlab: servers.gitlab(),
            token: servers.token().to_string(),
            project_id: None,
            project_path: None,
        }))
    }

    /// Get the GitLab container reference.
    pub fn gitlab(&self) -> &GitLabContainer {
        self.gitlab
    }

    /// Create a test project with a unique name.
    pub async fn create_test_project(&mut self) -> Result<()> {
        let project_name = format!("e2e-test-{}", Uuid::new_v4().to_string()[..8].to_string());

        let project = self
            .gitlab
            .create_project(&self.token, &project_name)
            .await
            .context("Failed to create test project")?;

        self.project_id = project["id"].as_u64();
        self.project_path = project["path_with_namespace"]
            .as_str()
            .map(|s| s.to_string());

        tracing::info!(
            "Created test project: {} (ID: {:?})",
            self.project_path.as_deref().unwrap_or("unknown"),
            self.project_id
        );

        Ok(())
    }

    /// Get the project path, creating a test project if needed.
    pub async fn ensure_project(&mut self) -> Result<String> {
        if self.project_path.is_none() {
            self.create_test_project().await?;
        }
        Ok(self.project_path.clone().unwrap())
    }

    /// Get the project ID, creating a test project if needed.
    pub async fn ensure_project_id(&mut self) -> Result<u64> {
        if self.project_id.is_none() {
            self.create_test_project().await?;
        }
        Ok(self.project_id.unwrap())
    }

    /// Cleanup test resources.
    pub async fn cleanup(self) -> Result<()> {
        // Delete test project if created
        if let Some(project_id) = self.project_id {
            let _ = self.gitlab.delete_project(&self.token, project_id).await;
        }

        // Shutdown MCP client (no-op for shared client - server stays running)
        self.client.shutdown().await?;

        Ok(())
    }
}

/// Builder for TestContext with custom options.
pub struct TestContextBuilder {
    transport: TransportKind,
    create_project: bool,
}

impl TestContextBuilder {
    /// Create a new builder with default options.
    pub fn new(transport: TransportKind) -> Self {
        Self {
            transport,
            create_project: false,
        }
    }

    /// Create a test project automatically.
    pub fn with_project(mut self) -> Self {
        self.create_project = true;
        self
    }

    /// Build the test context.
    ///
    /// Returns `None` if the requested transport is not available.
    pub async fn build(self) -> Result<Option<TestContext>> {
        let mut ctx = match TestContext::new(self.transport).await? {
            Some(ctx) => ctx,
            None => return Ok(None),
        };

        if self.create_project {
            ctx.create_test_project().await?;
        }

        Ok(Some(ctx))
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::SharedServers;

    #[test]
    fn test_config_generation() {
        let config = SharedServers::generate_config_for_test("http://localhost:8080", "test-token");
        assert!(config.contains("http://localhost:8080"));
        assert!(config.contains("test-token"));
        assert!(config.contains("all = "));
    }
}
