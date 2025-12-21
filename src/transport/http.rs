//! HTTP transport with Streamable HTTP
//!
//! Runs the MCP server over HTTP using the Streamable HTTP transport.

use crate::config::CorsMode;
use crate::server::GitLabMcpHandler;
use crate::util::bind_port_strict;
use axum::{Json, Router, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::{Instrument, info, info_span};

/// Default port for HTTP transport
pub const DEFAULT_HTTP_PORT: u16 = 20289;

/// Configuration for the HTTP server
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Address to bind to (e.g., "127.0.0.1:20289")
    pub bind: SocketAddr,
    /// Path for MCP endpoint (default: "/mcp")
    pub mcp_path: String,
    /// CORS mode (default: Permissive)
    pub cors: CorsMode,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], DEFAULT_HTTP_PORT)),
            mcp_path: "/mcp".to_string(),
            cors: CorsMode::default(),
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

/// Run the MCP server using HTTP transport with Streamable HTTP
///
/// This starts an HTTP server that handles MCP protocol messages using
/// the Streamable HTTP transport (replacement for deprecated SSE).
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
pub(crate) async fn run_http<F>(
    handler_factory: F,
    config: HttpConfig,
) -> anyhow::Result<CancellationToken>
where
    F: Fn() -> GitLabMcpHandler + Send + Sync + Clone + 'static,
{
    // Verify the port is available - fail if not (no fallback)
    let host = config.bind.ip().to_string();
    let preferred_port = config.bind.port();
    let actual_port = bind_port_strict(&host, preferred_port).await?;

    let bind_addr = SocketAddr::new(config.bind.ip(), actual_port);

    info!(
        "Starting GitLab MCP server with HTTP transport on {}",
        bind_addr
    );

    let ct = CancellationToken::new();

    // Create the Streamable HTTP service
    let service = StreamableHttpService::new(
        move || Ok(handler_factory()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // Build router with MCP service and health endpoint
    let router = Router::new()
        .nest_service(&config.mcp_path, service)
        .route("/health", get(health_handler));

    // Apply CORS layer based on config
    let router = match config.cors {
        CorsMode::Permissive => router.layer(CorsLayer::permissive()),
        CorsMode::Disabled => router,
    };

    // Bind and serve the router
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let server_ct = ct.child_token();
    let server = axum::serve(listener, router).with_graceful_shutdown({
        let ct = server_ct.clone();
        async move {
            ct.cancelled().await;
            info!("HTTP server cancelled");
        }
    });

    tokio::spawn(
        async move {
            if let Err(e) = server.await {
                tracing::error!(error = %e, "HTTP server shutdown with error");
            }
        }
        .instrument(info_span!("http-server", bind_address = %bind_addr)),
    );

    info!("HTTP server listening on http://{}", bind_addr);
    info!("  MCP endpoint: {}", config.mcp_path);
    info!("  Health endpoint: /health");

    Ok(ct)
}

/// Run the MCP server using HTTP transport and wait for shutdown
///
/// This is a convenience function that starts the server and waits
/// for a shutdown signal (Ctrl+C).
pub async fn run_http_blocking<F>(handler_factory: F, config: HttpConfig) -> anyhow::Result<()>
where
    F: Fn() -> GitLabMcpHandler + Send + Sync + Clone + 'static,
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

    info!("HTTP server stopped");
    Ok(())
}
