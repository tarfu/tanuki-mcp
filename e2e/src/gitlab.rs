//! GitLab CE container management for E2E tests.
//!
//! Provides utilities for:
//! - Starting GitLab CE Docker container
//! - Waiting for GitLab to be ready
//! - Creating Personal Access Tokens
//! - Managing test projects

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use tokio::process::Command;
use tokio::time::sleep;

/// Default GitLab root password for testing.
pub const DEFAULT_ROOT_PASSWORD: &str = "testpassword123!";

/// Default GitLab HTTP port.
pub const DEFAULT_GITLAB_PORT: u16 = 8080;

/// GitLab container configuration.
#[derive(Debug, Clone)]
pub struct GitLabConfig {
    /// The base URL for GitLab (e.g., "http://localhost:8080").
    pub base_url: String,
    /// The root password.
    pub root_password: String,
    /// Container name.
    pub container_name: String,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            base_url: format!("http://localhost:{}", DEFAULT_GITLAB_PORT),
            root_password: DEFAULT_ROOT_PASSWORD.to_string(),
            container_name: "tanuki-mcp-e2e-gitlab".to_string(),
        }
    }
}

impl GitLabConfig {
    /// Create config from a URL string (for environment variable usage).
    pub fn from_url(url: &str) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            ..Default::default()
        }
    }

    /// Get the GitLab base URL.
    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    /// Get the GitLab API URL.
    pub fn api_url(&self) -> String {
        format!("{}/api/v4", self.base_url)
    }
}

/// GitLab container manager.
pub struct GitLabContainer {
    config: GitLabConfig,
    client: Client,
}

impl GitLabContainer {
    /// Create a new GitLab container manager with default config.
    pub fn new() -> Self {
        Self::with_config(GitLabConfig::default())
    }

    /// Create a new GitLab container manager with custom config.
    pub fn with_config(config: GitLabConfig) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get the GitLab configuration.
    pub fn config(&self) -> &GitLabConfig {
        &self.config
    }

    /// Check if the GitLab container is running.
    pub async fn is_running(&self) -> bool {
        let output = Command::new("docker")
            .args([
                "inspect",
                "-f",
                "{{.State.Running}}",
                &self.config.container_name,
            ])
            .output()
            .await;

        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "true",
            Err(_) => false,
        }
    }

    /// Wait for GitLab to be healthy and ready.
    ///
    /// GitLab CE takes 3-5 minutes to fully start up.
    pub async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        // Use the login page as health check - the /-/health endpoints require explicit configuration
        let health_url = format!("{}/users/sign_in", self.config.base_url());

        // Use a shorter timeout for health checks to get faster feedback
        let health_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create health check client");

        tracing::info!("Waiting for GitLab to be ready at {}...", health_url);

        let mut attempt = 0;
        loop {
            attempt += 1;

            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timeout ({:?}) waiting for GitLab to be ready at {}",
                    timeout,
                    self.config.base_url()
                );
            }

            // Check login page - returns 200 when GitLab is ready
            match health_client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("GitLab is ready after {:?}", start.elapsed());
                    return Ok(());
                }
                Ok(resp) => {
                    tracing::debug!(
                        "Attempt {}: GitLab returned {} ({:?} elapsed)",
                        attempt,
                        resp.status(),
                        start.elapsed()
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        "Attempt {}: Connection error: {} ({:?} elapsed)",
                        attempt,
                        e,
                        start.elapsed()
                    );
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    /// Create a Personal Access Token for testing.
    ///
    /// Uses `docker exec` to run a Rails console command.
    pub async fn create_personal_access_token(
        &self,
        token_name: &str,
        token_value: &str,
    ) -> Result<String> {
        let ruby_script = format!(
            r#"
            user = User.find_by_username('root')
            token = user.personal_access_tokens.find_by(name: '{token_name}')
            if token
              token.destroy!
            end
            token = user.personal_access_tokens.create!(
              name: '{token_name}',
              scopes: ['api', 'read_api', 'read_repository', 'write_repository', 'read_user'],
              expires_at: 1.year.from_now
            )
            token.set_token('{token_value}')
            token.save!
            puts token.token
            "#,
            token_name = token_name,
            token_value = token_value
        );

        let output = Command::new("docker")
            .args([
                "exec",
                &self.config.container_name,
                "gitlab-rails",
                "runner",
                &ruby_script,
            ])
            .output()
            .await
            .context("Failed to execute gitlab-rails runner")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create PAT: {}", stderr);
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

        tracing::info!("Created PAT '{}' for root user", token_name);
        Ok(token)
    }

    /// Create a test project.
    pub async fn create_project(&self, token: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/projects", self.config.api_url());

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", token)
            .json(&serde_json::json!({
                "name": name,
                "visibility": "private",
                "initialize_with_readme": true
            }))
            .send()
            .await
            .context("Failed to send create project request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create project: {} - {}", status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse project response")
    }

    /// Delete a project by ID.
    pub async fn delete_project(&self, token: &str, project_id: u64) -> Result<()> {
        let url = format!("{}/projects/{}", self.config.api_url(), project_id);

        let response = self
            .client
            .delete(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .context("Failed to send delete project request")?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete project: {} - {}", status, body);
        }

        Ok(())
    }

    /// Get current user info (validates token).
    pub async fn get_current_user(&self, token: &str) -> Result<serde_json::Value> {
        let url = format!("{}/user", self.config.api_url());

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .context("Failed to send user request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get user: {} - {}", status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse user response")
    }

    /// Create an instance-level GitLab runner and return its authentication token.
    ///
    /// Uses `docker exec` to run a Rails console command.
    pub async fn create_runner(&self) -> Result<String> {
        let ruby_script = r#"
            # Delete existing e2e runner if present
            Ci::Runner.where(description: 'e2e-runner').destroy_all

            # Create new instance runner
            runner = Ci::Runner.create!(
              runner_type: :instance_type,
              description: 'e2e-runner',
              run_untagged: true,
              active: true
            )
            puts runner.token
        "#;

        let output = Command::new("docker")
            .args([
                "exec",
                &self.config.container_name,
                "gitlab-rails",
                "runner",
                ruby_script,
            ])
            .output()
            .await
            .context("Failed to execute gitlab-rails runner for creating runner")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create runner: {}", stderr);
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

        tracing::info!("Created instance runner 'e2e-runner'");
        Ok(token)
    }

    /// Register the GitLab runner container with the given token.
    ///
    /// Uses `docker exec` to run `gitlab-runner register`.
    pub async fn register_runner(&self, token: &str) -> Result<()> {
        const RUNNER_CONTAINER: &str = "tanuki-mcp-e2e-gitlab-runner";

        // First, unregister any existing runner config
        let _ = Command::new("docker")
            .args([
                "exec",
                RUNNER_CONTAINER,
                "gitlab-runner",
                "unregister",
                "--all-runners",
            ])
            .output()
            .await;

        // Register the runner
        let output = Command::new("docker")
            .args([
                "exec",
                RUNNER_CONTAINER,
                "gitlab-runner",
                "register",
                "--non-interactive",
                "--url",
                "http://gitlab:80",
                "--token",
                token,
                "--executor",
                "shell",
                "--description",
                "e2e-shell-runner",
            ])
            .output()
            .await
            .context("Failed to execute gitlab-runner register")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Failed to register runner: stderr={}, stdout={}",
                stderr,
                stdout
            );
        }

        tracing::info!("Registered runner 'e2e-shell-runner'");
        Ok(())
    }

    /// Start the GitLab runner daemon.
    ///
    /// Uses `docker exec` to run `gitlab-runner run` in the background.
    pub async fn start_runner(&self) -> Result<()> {
        const RUNNER_CONTAINER: &str = "tanuki-mcp-e2e-gitlab-runner";

        // Start the runner in the background using docker exec -d
        let output = Command::new("docker")
            .args(["exec", "-d", RUNNER_CONTAINER, "gitlab-runner", "run"])
            .output()
            .await
            .context("Failed to start gitlab-runner")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start runner: {}", stderr);
        }

        tracing::info!("Started runner daemon");

        // Wait a moment for runner to connect
        sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    /// Setup the GitLab runner: create token, register, and start.
    ///
    /// This is a convenience method that combines all runner setup steps.
    pub async fn setup_runner(&self) -> Result<()> {
        tracing::info!("Setting up GitLab runner...");

        let token = self.create_runner().await?;
        self.register_runner(&token).await?;
        self.start_runner().await?;

        tracing::info!("GitLab runner setup complete");
        Ok(())
    }
}

impl Default for GitLabContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_config_urls() {
        let config = GitLabConfig::default();
        assert_eq!(config.base_url(), "http://localhost:8080");
        assert_eq!(config.api_url(), "http://localhost:8080/api/v4");
    }

    #[test]
    fn test_from_url() {
        let config = GitLabConfig::from_url("http://localhost:9090");
        assert_eq!(config.base_url(), "http://localhost:9090");

        // Should trim trailing slash
        let config = GitLabConfig::from_url("http://example.com:8080/");
        assert_eq!(config.base_url(), "http://example.com:8080");
    }
}
