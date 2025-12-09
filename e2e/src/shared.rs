//! Shared MCP server management for E2E tests.
//!
//! This module provides shared MCP servers (one per transport type) that are
//! initialized once and reused across all tests. This dramatically reduces
//! test overhead by avoiding per-test server spawning.
//!
//! - HTTP transport (Streamable HTTP): Peer is Clone, tests can call concurrently
//! - Stdio transport: Mutex-protected, tests serialize access
//!
//! # Runtime Considerations
//!
//! Each `#[tokio::test]` creates its own runtime. To ensure the shared servers
//! persist across tests, we use a dedicated background runtime for the MCP
//! service tasks.

use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::{Peer, RoleClient, ServiceExt};
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransport;
use serde_json::Value;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

use crate::gitlab::{GitLabConfig, GitLabContainer};
use crate::transport::TransportKind;

/// Global shared servers singleton.
static SHARED_SERVERS: OnceLock<SharedServers> = OnceLock::new();

/// Dedicated runtime for MCP service tasks.
/// This runtime persists for the lifetime of the process, ensuring
/// service tasks don't get cancelled between tests.
static SERVICE_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Async initialization lock to prevent concurrent initialization.
static INIT_LOCK: TokioMutex<()> = TokioMutex::const_new(());

/// Get or create the dedicated service runtime.
fn get_service_runtime() -> &'static Runtime {
    SERVICE_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create service runtime")
    })
}

/// Get or initialize the shared servers.
///
/// This is safe to call from multiple tests concurrently - only the first
/// call will initialize the servers, subsequent calls return the same instance.
pub async fn get_shared_servers() -> &'static SharedServers {
    // Fast path: already initialized
    if let Some(servers) = SHARED_SERVERS.get() {
        return servers;
    }

    // Slow path: need to initialize (with lock to prevent races)
    let _guard = INIT_LOCK.lock().await;

    // Double-check after acquiring lock
    if let Some(servers) = SHARED_SERVERS.get() {
        return servers;
    }

    // Initialize in the service runtime so tasks persist
    let servers = get_service_runtime()
        .spawn(SharedServers::init())
        .await
        .expect("Service runtime task failed")
        .expect("Failed to initialize shared MCP servers");

    // This can fail if another thread initialized between our check and set,
    // but that's fine - we just use whatever was set
    let _ = SHARED_SERVERS.set(servers);
    SHARED_SERVERS.get().unwrap()
}

/// Container for shared servers (one per transport type).
pub struct SharedServers {
    http: Option<SharedHttpClient>, // None if MCP_HTTP_URL not set
    stdio: SharedStdioClient,
    gitlab: GitLabContainer,
    token: String,
    _config_dir: TempDir,
}

// Safety: SharedServers is safe to share across threads because:
// - http: Option<SharedHttpClient> contains Peer which is Clone + Send + Sync
// - stdio: SharedStdioClient uses Arc<TokioMutex<...>>
// - gitlab: GitLabContainer is Send + Sync
// - token: String is Send + Sync
// - _config_dir: TempDir is Send + Sync
unsafe impl Send for SharedServers {}
unsafe impl Sync for SharedServers {}

impl SharedServers {
    /// Initialize the shared servers.
    async fn init() -> Result<Self> {
        tracing::info!("Initializing shared MCP servers...");

        let gitlab_url = Self::get_gitlab_url()?;
        let token = Self::get_token()?;

        let gitlab = GitLabContainer::with_config(GitLabConfig::from_url(&gitlab_url));

        // Create config directory and file (needed for stdio)
        let config_dir = TempDir::new().context("Failed to create temp directory")?;
        let config_path = config_dir.path().join("config.toml");
        let config_content = Self::generate_config(&gitlab.config().base_url(), &token);
        std::fs::write(&config_path, &config_content).context("Failed to write config file")?;

        // Find binary (needed for stdio)
        let binary_path = Self::find_binary()?;

        // HTTP: only connect if MCP_HTTP_URL is set (server managed externally)
        let http = match std::env::var("MCP_HTTP_URL") {
            Ok(url) => {
                tracing::info!("Connecting to external HTTP server at {}", url);
                Some(SharedHttpClient::connect(&url).await?)
            }
            Err(_) => {
                tracing::info!("MCP_HTTP_URL not set, HTTP tests will be skipped");
                None
            }
        };

        // Stdio: always spawn locally
        let stdio = SharedStdioClient::init(&binary_path, &config_path).await?;

        tracing::info!("Shared MCP servers initialized successfully");

        Ok(Self {
            http,
            stdio,
            gitlab,
            token,
            _config_dir: config_dir,
        })
    }

    /// Get a peer for the specified transport type.
    ///
    /// Returns `None` for HTTP transport if `MCP_HTTP_URL` was not set.
    pub fn get_peer(&self, transport: TransportKind) -> Option<SharedPeer> {
        match transport {
            TransportKind::Http => self.http.as_ref().map(|h| SharedPeer::Http(h.peer())),
            TransportKind::Stdio => Some(SharedPeer::Stdio(self.stdio.clone())),
        }
    }

    /// Check if HTTP transport is available.
    pub fn has_http(&self) -> bool {
        self.http.is_some()
    }

    /// Get the shared GitLab container reference.
    pub fn gitlab(&self) -> &GitLabContainer {
        &self.gitlab
    }

    /// Get the shared token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get GitLab URL from environment variable.
    fn get_gitlab_url() -> Result<String> {
        std::env::var("GITLAB_URL").context(
            "GITLAB_URL environment variable not set. Run tests via 'task e2e' or set GITLAB_URL manually.",
        )
    }

    /// Get GitLab token from environment variable.
    fn get_token() -> Result<String> {
        std::env::var("GITLAB_TOKEN").context(
            "GITLAB_TOKEN environment variable not set. Run tests via 'task e2e' or set GITLAB_TOKEN manually.",
        )
    }

    /// Generate config file content.
    fn generate_config(gitlab_url: &str, token: &str) -> String {
        format!(
            r#"
[gitlab]
url = "{gitlab_url}"
token = "{token}"

[access_control]
all = "full"
"#,
            gitlab_url = gitlab_url,
            token = token
        )
    }

    /// Generate config file content (public for tests).
    pub fn generate_config_for_test(gitlab_url: &str, token: &str) -> String {
        Self::generate_config(gitlab_url, token)
    }

    /// Find the tanuki-mcp binary.
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
        anyhow::bail!(
            "tanuki-mcp binary not found. Please run 'cargo build' or 'cargo build --release' first."
        )
    }
}

/// Shared HTTP client - peer can be cloned freely for concurrent use.
///
/// Connects to an external HTTP server specified by `MCP_HTTP_URL`.
/// The server must be started externally (e.g., via Taskfile).
pub struct SharedHttpClient {
    peer: Peer<RoleClient>,
}

impl SharedHttpClient {
    /// Connect to an external HTTP server.
    ///
    /// The URL should be the MCP endpoint (e.g., `http://127.0.0.1:20399/mcp`).
    async fn connect(url: &str) -> Result<Self> {
        let transport = StreamableHttpClientTransport::from_uri(url.to_string());

        let running_service =
            ().serve(transport)
                .await
                .context("Failed to start MCP client service")?;

        let peer = running_service.peer().clone();

        // Spawn the service task in the background - it will run until cancelled
        tokio::spawn(async move {
            let _ = running_service.waiting().await;
        });

        Ok(Self { peer })
    }

    /// Get a cloned peer for concurrent use.
    pub fn peer(&self) -> Peer<RoleClient> {
        self.peer.clone()
    }
}

/// Shared Stdio client - must be accessed via mutex for serialized access.
#[derive(Clone)]
pub struct SharedStdioClient {
    inner: std::sync::Arc<TokioMutex<SharedStdioInner>>,
}

struct SharedStdioInner {
    peer: Peer<RoleClient>,
}

impl SharedStdioClient {
    /// Initialize the shared Stdio server.
    async fn init(binary_path: &PathBuf, config_path: &PathBuf) -> Result<Self> {
        tracing::debug!("Starting shared Stdio server");

        let mut cmd = Command::new(binary_path);
        cmd.env("TANUKI_MCP_CONFIG", config_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        let transport =
            TokioChildProcess::new(cmd).context("Failed to create child process transport")?;

        let running_service =
            ().serve(transport)
                .await
                .context("Failed to start MCP client service")?;

        let peer = running_service.peer().clone();

        // Spawn the service task in the background - it will run until cancelled
        // The task is spawned in the current runtime (service runtime)
        tokio::spawn(async move {
            let _ = running_service.waiting().await;
        });

        Ok(Self {
            inner: std::sync::Arc::new(TokioMutex::new(SharedStdioInner { peer })),
        })
    }

    /// Call a tool (acquires mutex internally).
    pub async fn call_tool(&self, param: CallToolRequestParam) -> Result<CallToolResult> {
        let guard = self.inner.lock().await;
        guard
            .peer
            .call_tool(param)
            .await
            .context("Failed to call tool")
    }

    /// List tools (acquires mutex internally).
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        let guard = self.inner.lock().await;
        guard
            .peer
            .list_tools(Default::default())
            .await
            .context("Failed to list tools")
    }

    /// List all tools (acquires mutex internally).
    pub async fn list_all_tools(&self) -> Result<Vec<rmcp::model::Tool>> {
        let guard = self.inner.lock().await;
        guard
            .peer
            .list_all_tools()
            .await
            .context("Failed to list all tools")
    }
}

/// A peer handle that abstracts over transport types.
pub enum SharedPeer {
    /// HTTP peer - can be used concurrently (Peer is Clone).
    Http(Peer<RoleClient>),
    /// Stdio peer - serialized access via mutex.
    Stdio(SharedStdioClient),
}

impl SharedPeer {
    /// Call a tool with the given name and arguments.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult> {
        let args = match arguments {
            Value::Object(map) => Some(map),
            Value::Null => None,
            _ => anyhow::bail!("Arguments must be a JSON object or null"),
        };

        let param = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: args,
        };

        match self {
            SharedPeer::Http(peer) => peer
                .call_tool(param)
                .await
                .context(format!("Failed to call tool: {}", name)),
            SharedPeer::Stdio(stdio) => stdio.call_tool(param).await,
        }
    }

    /// List available tools.
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        match self {
            SharedPeer::Http(peer) => peer
                .list_tools(Default::default())
                .await
                .context("Failed to list tools"),
            SharedPeer::Stdio(stdio) => stdio.list_tools().await,
        }
    }

    /// List all tools (handles pagination).
    pub async fn list_all_tools(&self) -> Result<Vec<rmcp::model::Tool>> {
        match self {
            SharedPeer::Http(peer) => peer
                .list_all_tools()
                .await
                .context("Failed to list all tools"),
            SharedPeer::Stdio(stdio) => stdio.list_all_tools().await,
        }
    }
}

/// Wrapper providing McpClient-compatible interface for tests.
///
/// This is a drop-in replacement for McpClient that uses shared servers.
pub struct SharedMcpClient {
    peer: SharedPeer,
    /// The transport kind used.
    pub kind: TransportKind,
}

impl SharedMcpClient {
    /// Create a new shared client wrapper.
    pub fn new(peer: SharedPeer, kind: TransportKind) -> Self {
        Self { peer, kind }
    }

    /// List all available tools.
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        self.peer.list_tools().await
    }

    /// List all tools (handles pagination).
    pub async fn list_all_tools(&self) -> Result<Vec<rmcp::model::Tool>> {
        self.peer.list_all_tools().await
    }

    /// Call a tool with the given name and arguments.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult> {
        self.peer.call_tool(name, arguments).await
    }

    /// Call a tool and extract the text content from the result.
    pub async fn call_tool_text(&self, name: &str, arguments: Value) -> Result<String> {
        let result = self.call_tool(name, arguments).await?;

        let mut text_parts = Vec::new();
        for content in result.content {
            if let rmcp::model::RawContent::Text(text_content) = content.raw {
                text_parts.push(text_content.text);
            }
        }

        Ok(text_parts.join("\n"))
    }

    /// Call a tool and parse the result as JSON.
    pub async fn call_tool_json(&self, name: &str, arguments: Value) -> Result<Value> {
        let text = self.call_tool_text(name, arguments).await?;
        serde_json::from_str(&text).context("Failed to parse tool result as JSON")
    }

    /// Shutdown the client (no-op for shared client).
    ///
    /// The shared server stays running for other tests.
    pub async fn shutdown(self) -> Result<()> {
        // No-op - server stays alive for other tests
        Ok(())
    }
}
