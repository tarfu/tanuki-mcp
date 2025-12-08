//! MCP transport abstraction for E2E tests.
//!
//! Provides a unified interface for testing tanuki-mcp via different transports:
//! - Stdio: Spawns tanuki-mcp as a child process
//! - HTTP/SSE: Connects to tanuki-mcp running in HTTP mode

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::{Peer, RoleClient, RunningService, ServiceExt};
use rmcp::transport::SseClientTransport;
use rmcp::transport::child_process::TokioChildProcess;
use serde_json::Value;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// The transport kind for E2E tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    /// Stdio transport - spawns tanuki-mcp as a child process.
    Stdio,
    /// HTTP/SSE transport - connects to a running tanuki-mcp HTTP server.
    Http,
}

impl std::fmt::Display for TransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportKind::Stdio => write!(f, "stdio"),
            TransportKind::Http => write!(f, "http"),
        }
    }
}

/// A connected MCP client that can call tools.
pub struct McpClient {
    /// The transport kind used.
    pub kind: TransportKind,
    /// The running service (for stdio transport).
    running_service: Option<RunningService<RoleClient, ()>>,
    /// The peer for making requests (for SSE transport).
    sse_peer: Option<Peer<RoleClient>>,
    /// Child process for HTTP server (needs to be kept alive).
    http_server_process: Option<Child>,
}

impl McpClient {
    /// Create a new MCP client using the stdio transport.
    ///
    /// Spawns tanuki-mcp as a child process and connects via stdin/stdout.
    pub async fn new_stdio(binary_path: &Path, config_path: &Path) -> Result<Self> {
        let mut cmd = Command::new(binary_path);
        cmd.env("TANUKI_MCP_CONFIG", config_path);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::inherit());

        let transport =
            TokioChildProcess::new(cmd).context("Failed to create child process transport")?;

        let running_service =
            ().serve(transport)
                .await
                .context("Failed to start MCP client service")?;

        Ok(Self {
            kind: TransportKind::Stdio,
            running_service: Some(running_service),
            sse_peer: None,
            http_server_process: None,
        })
    }

    /// Create a new MCP client using the HTTP/SSE transport.
    ///
    /// First spawns tanuki-mcp in HTTP mode, waits for it to be ready,
    /// then connects via SSE.
    pub async fn new_http(binary_path: &Path, config_path: &Path, http_port: u16) -> Result<Self> {
        // Start tanuki-mcp in HTTP mode
        let mut cmd = Command::new(binary_path);
        cmd.arg("--transport");
        cmd.arg("http");
        cmd.arg("--http-port");
        cmd.arg(http_port.to_string());
        cmd.env("TANUKI_MCP_CONFIG", config_path);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let child = cmd.spawn().context("Failed to spawn HTTP server")?;

        // Wait for server to be ready
        let sse_url = format!("http://127.0.0.1:{}/sse", http_port);
        Self::wait_for_server(&sse_url, Duration::from_secs(30)).await?;

        // Connect via SSE
        let transport = SseClientTransport::start(sse_url)
            .await
            .context("Failed to create SSE transport")?;

        let running_service =
            ().serve(transport)
                .await
                .context("Failed to connect to MCP server via SSE")?;

        Ok(Self {
            kind: TransportKind::Http,
            running_service: Some(running_service),
            sse_peer: None,
            http_server_process: Some(child),
        })
    }

    /// Wait for the HTTP server to be ready.
    async fn wait_for_server(url: &str, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        // Extract host:port from URL
        let url_parsed = url::Url::parse(url).context("Invalid URL")?;
        let host = url_parsed.host_str().unwrap_or("127.0.0.1");
        let port = url_parsed.port().unwrap_or(80);
        let addr = format!("{}:{}", host, port);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for HTTP server at {}", url);
            }

            // Try to establish a TCP connection to check if server is listening
            match tokio::net::TcpStream::connect(&addr).await {
                Ok(_) => {
                    // Give the server a moment to be fully ready
                    sleep(Duration::from_millis(100)).await;
                    return Ok(());
                }
                Err(_) => {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get the peer for making MCP requests.
    fn peer(&self) -> &Peer<RoleClient> {
        if let Some(ref service) = self.running_service {
            service.peer()
        } else if let Some(ref peer) = self.sse_peer {
            peer
        } else {
            panic!("No peer available")
        }
    }

    /// List all available tools.
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        self.peer()
            .list_tools(Default::default())
            .await
            .context("Failed to list tools")
    }

    /// List all tools (handles pagination).
    pub async fn list_all_tools(&self) -> Result<Vec<rmcp::model::Tool>> {
        self.peer()
            .list_all_tools()
            .await
            .context("Failed to list all tools")
    }

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

        self.peer()
            .call_tool(param)
            .await
            .context(format!("Failed to call tool: {}", name))
    }

    /// Call a tool and extract the text content from the result.
    pub async fn call_tool_text(&self, name: &str, arguments: Value) -> Result<String> {
        let result = self.call_tool(name, arguments).await?;

        // Extract text from content blocks
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

    /// Shutdown the client and cleanup resources.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(service) = self.running_service.take() {
            let _ = service.cancel().await;
        }

        if let Some(mut child) = self.http_server_process.take() {
            let _ = child.kill().await;
        }

        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Best effort cleanup - kill HTTP server if still running
        if let Some(ref mut child) = self.http_server_process {
            // Send kill signal
            let _ = child.start_kill();

            // Wait for process to actually exit (with timeout)
            // We need to poll try_wait since we can't use async in Drop
            for _ in 0..50 {
                // 50 * 20ms = 1 second max
                match child.try_wait() {
                    Ok(Some(_)) => break, // Process exited
                    Ok(None) => {
                        // Still running, wait a bit
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    Err(_) => break, // Error, give up
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_kind_display() {
        assert_eq!(TransportKind::Stdio.to_string(), "stdio");
        assert_eq!(TransportKind::Http.to_string(), "http");
    }
}
