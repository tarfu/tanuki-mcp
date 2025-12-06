//! HTTP/SSE transport
//!
//! Runs the MCP server over HTTP with Server-Sent Events (SSE).

use crate::server::GitLabMcpHandler;
use crate::util::find_available_port;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tracing::info;

/// Default port for HTTP/SSE transport
pub const DEFAULT_HTTP_PORT: u16 = 20289;

/// Configuration for the HTTP/SSE server
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Address to bind to (e.g., "127.0.0.1:20289")
    pub bind: SocketAddr,
    /// Path for SSE endpoint (default: "/sse")
    pub sse_path: String,
    /// Path for message posting (default: "/message")
    pub post_path: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], DEFAULT_HTTP_PORT)),
            sse_path: "/sse".to_string(),
            post_path: "/message".to_string(),
        }
    }
}

impl HttpConfig {
    /// Create a new HTTP config with the specified bind address
    pub fn new(bind: SocketAddr) -> Self {
        Self {
            bind,
            ..Default::default()
        }
    }

    /// Create config from host and port strings
    pub fn from_host_port(host: &str, port: u16) -> Result<Self, std::net::AddrParseError> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
        Ok(Self::new(addr))
    }
}

/// Run the MCP server using HTTP/SSE transport
///
/// This starts an HTTP server that accepts SSE connections and handles
/// MCP protocol messages over the SSE channel.
///
/// Port discovery is used to find an available port if the configured port is taken.
///
/// # Arguments
/// * `handler_factory` - A function that creates a new handler for each connection
/// * `config` - HTTP server configuration
///
/// # Returns
/// A cancellation token that can be used to stop the server
pub async fn run_http<F>(
    handler_factory: F,
    config: HttpConfig,
) -> anyhow::Result<CancellationToken>
where
    F: Fn() -> GitLabMcpHandler + Send + Sync + 'static,
{
    // Find an available port using port discovery
    let host = config.bind.ip().to_string();
    let preferred_port = config.bind.port();
    let actual_port = find_available_port(&host, preferred_port).await?;

    let bind_addr = SocketAddr::new(config.bind.ip(), actual_port);

    info!(
        "Starting GitLab MCP server with HTTP/SSE transport on {}",
        bind_addr
    );

    let ct = CancellationToken::new();

    let sse_config = SseServerConfig {
        bind: bind_addr,
        sse_path: config.sse_path,
        post_path: config.post_path,
        ct: ct.clone(),
    };

    let sse_server = SseServer::serve_with_config(sse_config).await?;

    info!(
        "HTTP/SSE server listening on http://{}",
        sse_server.config.bind
    );
    info!("  SSE endpoint: {}", sse_server.config.sse_path);
    info!("  Message endpoint: {}", sse_server.config.post_path);

    // Use the with_service method to handle incoming connections
    let server_ct = sse_server.with_service(handler_factory);

    Ok(server_ct)
}

/// Run the MCP server using HTTP/SSE transport and wait for shutdown
///
/// This is a convenience function that starts the server and waits
/// for a shutdown signal (Ctrl+C).
pub async fn run_http_blocking<F>(handler_factory: F, config: HttpConfig) -> anyhow::Result<()>
where
    F: Fn() -> GitLabMcpHandler + Send + Sync + 'static,
{
    let ct = run_http(handler_factory, config).await?;

    info!("Press Ctrl+C to stop the server");

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = ct.cancelled() => {
            info!("Server cancelled");
        }
    }

    // Cancel the server
    ct.cancel();

    info!("HTTP/SSE server stopped");
    Ok(())
}
