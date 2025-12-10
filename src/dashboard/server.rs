//! Dashboard HTTP server
//!
//! Serves the dashboard web interface and API endpoints.

use crate::config::AppConfig;
use crate::dashboard::metrics::{DashboardMetrics, MetricsSnapshot};
use crate::util::find_available_port;
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

/// Default port for the dashboard server
pub const DEFAULT_DASHBOARD_PORT: u16 = 19892;

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Address to bind the dashboard server
    pub bind: SocketAddr,
    /// Enable dashboard (default: true)
    pub enabled: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], DEFAULT_DASHBOARD_PORT)),
            enabled: true,
        }
    }
}

impl DashboardConfig {
    /// Create config from host and port
    pub fn new(host: &str, port: u16) -> Result<Self, std::net::AddrParseError> {
        let bind: SocketAddr = format!("{}:{}", host, port).parse()?;
        Ok(Self {
            bind,
            enabled: true,
        })
    }
}

/// Shared state for dashboard handlers
#[derive(Clone)]
pub struct DashboardState {
    pub metrics: Arc<DashboardMetrics>,
    pub config: Arc<AppConfig>,
}

/// Configuration info for the API
#[derive(Serialize)]
struct ConfigInfo {
    server_name: String,
    server_version: String,
    gitlab_url: String,
    transport_mode: String,
    access_level: String,
    tool_count: usize,
}

/// Run the dashboard server
///
/// Port discovery is used to find an available port if the configured port is taken.
pub async fn run_dashboard(
    config: DashboardConfig,
    metrics: Arc<DashboardMetrics>,
    app_config: Arc<AppConfig>,
    tool_count: usize,
) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Dashboard is disabled");
        return Ok(());
    }

    // Find an available port using port discovery
    let host = config.bind.ip().to_string();
    let preferred_port = config.bind.port();
    let actual_port = find_available_port(&host, preferred_port).await?;

    let bind_addr = SocketAddr::new(config.bind.ip(), actual_port);

    let state = DashboardState {
        metrics,
        config: app_config,
    };

    // Build router
    let app = Router::new()
        .route("/", get(dashboard_html))
        .route("/api/metrics", get(api_metrics))
        .route("/api/config", get(move |s| api_config(s, tool_count)))
        .route("/assets/style.css", get(serve_css))
        .route("/assets/app.js", get(serve_js))
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await?;
    info!("Dashboard server running at http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the main dashboard HTML page
async fn dashboard_html() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

/// API endpoint for metrics
async fn api_metrics(State(state): State<DashboardState>) -> Json<MetricsSnapshot> {
    Json(state.metrics.snapshot())
}

/// API endpoint for configuration
async fn api_config(State(state): State<DashboardState>, tool_count: usize) -> Json<ConfigInfo> {
    let config = &state.config;
    Json(ConfigInfo {
        server_name: config.server.name.clone(),
        server_version: config.server.version.clone(),
        gitlab_url: config.gitlab.url.clone(),
        transport_mode: format!("{:?}", config.server.transport),
        access_level: format!("{:?}", config.access_control.all),
        tool_count,
    })
}

/// Serve CSS
async fn serve_css() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        DASHBOARD_CSS,
    )
        .into_response()
}

/// Serve JavaScript
async fn serve_js() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        DASHBOARD_JS,
    )
        .into_response()
}

/// Dashboard HTML template
const DASHBOARD_HTML: &str = include_str!("../../assets/dashboard/index.html");

/// Dashboard CSS styles
const DASHBOARD_CSS: &str = include_str!("../../assets/dashboard/style.css");

/// Dashboard JavaScript
const DASHBOARD_JS: &str = include_str!("../../assets/dashboard/app.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.bind.port(), DEFAULT_DASHBOARD_PORT);
        assert!(config.enabled);
    }

    #[test]
    fn test_dashboard_config_new() {
        let config = DashboardConfig::new("127.0.0.1", 8080).unwrap();
        assert_eq!(config.bind.port(), 8080);
    }
}
