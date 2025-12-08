//! HTTP/SSE transport
//!
//! Runs the MCP server over HTTP with Server-Sent Events (SSE).

use crate::server::GitLabMcpHandler;
use crate::util::bind_port_strict;
use axum::{Json, routing::get};
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info, info_span};

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

/// Health check endpoint handler
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// Run the MCP server using HTTP/SSE transport
///
/// This starts an HTTP server that accepts SSE connections and handles
/// MCP protocol messages over the SSE channel.
///
/// The server will fail if the configured port is already in use. This is
/// intentional to ensure clients can reliably connect to the expected port.
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
    // Verify the port is available - fail if not (no fallback)
    let host = config.bind.ip().to_string();
    let preferred_port = config.bind.port();
    let actual_port = bind_port_strict(&host, preferred_port).await?;

    let bind_addr = SocketAddr::new(config.bind.ip(), actual_port);

    info!(
        "Starting GitLab MCP server with HTTP/SSE transport on {}",
        bind_addr
    );

    let ct = CancellationToken::new();

    let sse_config = SseServerConfig {
        bind: bind_addr,
        sse_path: config.sse_path.clone(),
        post_path: config.post_path.clone(),
        ct: ct.clone(),
        sse_keep_alive: None,
    };

    // Create SSE server and router, then add health endpoint
    let (sse_server, sse_router) = SseServer::new(sse_config);

    // Add health endpoint to the router
    let router = sse_router.route("/health", get(health_handler));

    // Bind and serve the router
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let server_ct = ct.child_token();
    let server = axum::serve(listener, router).with_graceful_shutdown({
        let ct = server_ct.clone();
        async move {
            ct.cancelled().await;
            info!("SSE server cancelled");
        }
    });

    tokio::spawn(
        async move {
            if let Err(e) = server.await {
                tracing::error!(error = %e, "SSE server shutdown with error");
            }
        }
        .instrument(info_span!("sse-server", bind_address = %bind_addr)),
    );

    info!("HTTP/SSE server listening on http://{}", bind_addr);
    info!("  SSE endpoint: {}", config.sse_path);
    info!("  Message endpoint: {}", config.post_path);
    info!("  Health endpoint: /health");

    // Use the with_service method to handle incoming connections
    let _service_ct = sse_server.with_service(handler_factory);

    Ok(ct)
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
