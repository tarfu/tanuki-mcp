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
const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GitLab MCP Dashboard</title>
    <link rel="stylesheet" href="/assets/style.css">
</head>
<body>
    <header>
        <div class="header-content">
            <h1>
                <svg class="logo" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z"/>
                </svg>
                GitLab MCP Dashboard
            </h1>
            <div class="status-indicator">
                <span class="status-dot"></span>
                <span id="status-text">Connected</span>
            </div>
        </div>
    </header>

    <main>
        <!-- Overview Cards -->
        <section class="overview-cards">
            <div class="card stat-card">
                <div class="stat-icon requests-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                    </svg>
                </div>
                <div class="stat-content">
                    <span class="stat-value" id="total-requests">0</span>
                    <span class="stat-label">Total Requests</span>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon errors-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <line x1="12" y1="8" x2="12" y2="12"/>
                        <line x1="12" y1="16" x2="12.01" y2="16"/>
                    </svg>
                </div>
                <div class="stat-content">
                    <span class="stat-value" id="total-errors">0</span>
                    <span class="stat-label">Errors</span>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon rate-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
                        <polyline points="17 6 23 6 23 12"/>
                    </svg>
                </div>
                <div class="stat-content">
                    <span class="stat-value" id="requests-rate">0</span>
                    <span class="stat-label">Requests/min</span>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon uptime-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                </div>
                <div class="stat-content">
                    <span class="stat-value" id="uptime">0s</span>
                    <span class="stat-label">Uptime</span>
                </div>
            </div>
        </section>

        <!-- Configuration Section -->
        <section class="config-section">
            <div class="card">
                <h2>Configuration</h2>
                <div class="config-grid" id="config-grid">
                    <div class="config-item">
                        <span class="config-label">Server Name</span>
                        <span class="config-value" id="server-name">-</span>
                    </div>
                    <div class="config-item">
                        <span class="config-label">Version</span>
                        <span class="config-value" id="server-version">-</span>
                    </div>
                    <div class="config-item">
                        <span class="config-label">GitLab URL</span>
                        <span class="config-value" id="gitlab-url">-</span>
                    </div>
                    <div class="config-item">
                        <span class="config-label">Transport</span>
                        <span class="config-value" id="transport-mode">-</span>
                    </div>
                    <div class="config-item">
                        <span class="config-label">Access Level</span>
                        <span class="config-value" id="access-level">-</span>
                    </div>
                    <div class="config-item">
                        <span class="config-label">Tools Available</span>
                        <span class="config-value" id="tool-count">-</span>
                    </div>
                </div>
            </div>
        </section>

        <!-- Two Column Layout -->
        <div class="two-column">
            <!-- Projects Section -->
            <section class="card">
                <h2>Projects Accessed</h2>
                <div class="table-container">
                    <table id="projects-table">
                        <thead>
                            <tr>
                                <th>Project</th>
                                <th>Access Count</th>
                                <th>Tools Used</th>
                                <th>Last Accessed</th>
                            </tr>
                        </thead>
                        <tbody id="projects-body">
                            <tr class="empty-row">
                                <td colspan="4">No projects accessed yet</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </section>

            <!-- Tools Section -->
            <section class="card">
                <h2>Tool Usage</h2>
                <div class="table-container">
                    <table id="tools-table">
                        <thead>
                            <tr>
                                <th>Tool</th>
                                <th>Calls</th>
                                <th>Errors</th>
                                <th>Avg Duration</th>
                            </tr>
                        </thead>
                        <tbody id="tools-body">
                            <tr class="empty-row">
                                <td colspan="4">No tools called yet</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </section>
        </div>

        <!-- Categories Section -->
        <section class="card">
            <h2>Usage by Category</h2>
            <div class="categories-grid" id="categories-grid">
                <div class="category-placeholder">No category data yet</div>
            </div>
        </section>

        <!-- Recent Requests Section -->
        <section class="card">
            <h2>Recent Requests</h2>
            <div class="table-container">
                <table id="recent-table">
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>Tool</th>
                            <th>Project</th>
                            <th>Status</th>
                            <th>Duration</th>
                        </tr>
                    </thead>
                    <tbody id="recent-body">
                        <tr class="empty-row">
                            <td colspan="5">No recent requests</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </section>
    </main>

    <footer>
        <p>GitLab MCP Server Dashboard &bull; Auto-refreshes every 2 seconds</p>
    </footer>

    <script src="/assets/app.js"></script>
</body>
</html>
"##;

/// Dashboard CSS styles
const DASHBOARD_CSS: &str = r##"
:root {
    --bg-primary: #0d1117;
    --bg-secondary: #161b22;
    --bg-tertiary: #21262d;
    --border-color: #30363d;
    --text-primary: #c9d1d9;
    --text-secondary: #8b949e;
    --text-muted: #6e7681;
    --accent-blue: #58a6ff;
    --accent-green: #3fb950;
    --accent-orange: #d29922;
    --accent-red: #f85149;
    --accent-purple: #a371f7;
    --gitlab-orange: #fc6d26;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    line-height: 1.5;
    min-height: 100vh;
}

header {
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    padding: 1rem 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
}

.header-content {
    max-width: 1400px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

header h1 {
    font-size: 1.5rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.logo {
    width: 28px;
    height: 28px;
    color: var(--gitlab-orange);
}

.status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--accent-green);
    animation: pulse 2s infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

main {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
}

.card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}

.card h2 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary);
}

/* Overview Cards */
.overview-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.stat-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.25rem;
}

.stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.stat-icon svg {
    width: 24px;
    height: 24px;
}

.requests-icon {
    background-color: rgba(88, 166, 255, 0.1);
    color: var(--accent-blue);
}

.errors-icon {
    background-color: rgba(248, 81, 73, 0.1);
    color: var(--accent-red);
}

.rate-icon {
    background-color: rgba(63, 185, 80, 0.1);
    color: var(--accent-green);
}

.uptime-icon {
    background-color: rgba(163, 113, 247, 0.1);
    color: var(--accent-purple);
}

.stat-content {
    display: flex;
    flex-direction: column;
}

.stat-value {
    font-size: 1.75rem;
    font-weight: 600;
    line-height: 1.2;
}

.stat-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
}

/* Config Section */
.config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
}

.config-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.config-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
}

.config-value {
    font-size: 0.9375rem;
    color: var(--text-primary);
    word-break: break-all;
}

/* Two Column Layout */
.two-column {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 1.5rem;
}

.two-column .card {
    margin-bottom: 0;
}

/* Tables */
.table-container {
    overflow-x: auto;
}

table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
}

thead {
    background-color: var(--bg-tertiary);
}

th, td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
}

th {
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

tbody tr:hover {
    background-color: var(--bg-tertiary);
}

.empty-row td {
    text-align: center;
    color: var(--text-muted);
    padding: 2rem;
}

.status-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
}

.status-success {
    background-color: rgba(63, 185, 80, 0.15);
    color: var(--accent-green);
}

.status-error {
    background-color: rgba(248, 81, 73, 0.15);
    color: var(--accent-red);
}

/* Categories Grid */
.categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
}

.category-card {
    background-color: var(--bg-tertiary);
    border-radius: 6px;
    padding: 1rem;
    text-align: center;
}

.category-name {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
    text-transform: capitalize;
}

.category-count {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
}

.category-errors {
    font-size: 0.75rem;
    color: var(--accent-red);
    margin-top: 0.25rem;
}

.category-placeholder {
    grid-column: 1 / -1;
    text-align: center;
    color: var(--text-muted);
    padding: 2rem;
}

/* Footer */
footer {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
    font-size: 0.875rem;
}

/* Responsive */
@media (max-width: 768px) {
    main {
        padding: 1rem;
    }

    .two-column {
        grid-template-columns: 1fr;
    }

    .overview-cards {
        grid-template-columns: repeat(2, 1fr);
    }

    .stat-card {
        flex-direction: column;
        text-align: center;
    }
}
"##;

/// Dashboard JavaScript
const DASHBOARD_JS: &str = r##"
// Dashboard state
let lastData = null;

// Format uptime
function formatUptime(seconds) {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
}

// Format timestamp
function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString();
}

// Format relative time
function formatRelativeTime(timestamp) {
    if (!timestamp) return '-';
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
}

// Update overview stats
function updateOverview(data) {
    document.getElementById('total-requests').textContent = data.total_requests.toLocaleString();
    document.getElementById('total-errors').textContent = data.total_errors.toLocaleString();
    document.getElementById('requests-rate').textContent = data.requests_per_minute.toFixed(1);
    document.getElementById('uptime').textContent = formatUptime(data.uptime_secs);
}

// Update config section
function updateConfig(config) {
    document.getElementById('server-name').textContent = config.server_name;
    document.getElementById('server-version').textContent = config.server_version;
    document.getElementById('gitlab-url').textContent = config.gitlab_url;
    document.getElementById('transport-mode').textContent = config.transport_mode;
    document.getElementById('access-level').textContent = config.access_level;
    document.getElementById('tool-count').textContent = config.tool_count;
}

// Update projects table
function updateProjects(projects) {
    const tbody = document.getElementById('projects-body');

    if (projects.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="4">No projects accessed yet</td></tr>';
        return;
    }

    tbody.innerHTML = projects.map(p => `
        <tr>
            <td><code>${escapeHtml(p.name)}</code></td>
            <td>${p.access_count}</td>
            <td>${p.tools_used.slice(0, 3).map(t => t.tool).join(', ')}${p.tools_used.length > 3 ? '...' : ''}</td>
            <td>${formatRelativeTime(p.last_accessed)}</td>
        </tr>
    `).join('');
}

// Update tools table
function updateTools(tools) {
    const tbody = document.getElementById('tools-body');

    if (tools.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="4">No tools called yet</td></tr>';
        return;
    }

    tbody.innerHTML = tools.slice(0, 15).map(t => `
        <tr>
            <td><code>${escapeHtml(t.name)}</code></td>
            <td>${t.call_count}</td>
            <td>${t.error_count > 0 ? `<span class="status-badge status-error">${t.error_count}</span>` : '-'}</td>
            <td>${t.avg_duration_ms}ms</td>
        </tr>
    `).join('');
}

// Update categories grid
function updateCategories(categories) {
    const grid = document.getElementById('categories-grid');

    if (categories.length === 0) {
        grid.innerHTML = '<div class="category-placeholder">No category data yet</div>';
        return;
    }

    grid.innerHTML = categories.map(c => `
        <div class="category-card">
            <div class="category-name">${escapeHtml(c.name)}</div>
            <div class="category-count">${c.call_count}</div>
            ${c.error_count > 0 ? `<div class="category-errors">${c.error_count} errors</div>` : ''}
        </div>
    `).join('');
}

// Update recent requests table
function updateRecent(requests) {
    const tbody = document.getElementById('recent-body');

    if (requests.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="5">No recent requests</td></tr>';
        return;
    }

    // Show most recent first
    const reversed = [...requests].reverse();

    tbody.innerHTML = reversed.slice(0, 20).map(r => `
        <tr>
            <td>${formatTime(r.timestamp)}</td>
            <td><code>${escapeHtml(r.tool)}</code></td>
            <td>${r.project ? `<code>${escapeHtml(r.project)}</code>` : '-'}</td>
            <td><span class="status-badge ${r.success ? 'status-success' : 'status-error'}">${r.success ? 'OK' : 'Error'}</span></td>
            <td>${r.duration_ms}ms</td>
        </tr>
    `).join('');
}

// Escape HTML to prevent XSS
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Fetch and update metrics
async function fetchMetrics() {
    try {
        const response = await fetch('/api/metrics');
        const data = await response.json();

        updateOverview(data);
        updateProjects(data.projects);
        updateTools(data.tools);
        updateCategories(data.categories);
        updateRecent(data.recent_requests);

        document.getElementById('status-text').textContent = 'Connected';
        document.querySelector('.status-dot').style.backgroundColor = 'var(--accent-green)';

        lastData = data;
    } catch (error) {
        console.error('Failed to fetch metrics:', error);
        document.getElementById('status-text').textContent = 'Disconnected';
        document.querySelector('.status-dot').style.backgroundColor = 'var(--accent-red)';
    }
}

// Fetch config once
async function fetchConfig() {
    try {
        const response = await fetch('/api/config');
        const config = await response.json();
        updateConfig(config);
    } catch (error) {
        console.error('Failed to fetch config:', error);
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    fetchConfig();
    fetchMetrics();

    // Refresh every 2 seconds
    setInterval(fetchMetrics, 2000);
});
"##;

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
