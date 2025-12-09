//! GitLab MCP Server
//!
//! A Model Context Protocol server for GitLab with fine-grained access control.

use clap::Parser;
use std::sync::Arc;
use tanuki_mcp::{
    access_control::AccessResolver,
    auth::create_auth_provider,
    config::{AppConfig, TransportMode, load_config},
    dashboard::{DEFAULT_DASHBOARD_PORT, DashboardConfig, DashboardMetrics, run_dashboard},
    gitlab::GitLabClient,
    server::GitLabMcpHandler,
    transport::{DEFAULT_HTTP_PORT, HttpConfig, run_http_blocking, run_stdio},
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// GitLab MCP Server - Fine-grained access control for GitLab via MCP
#[derive(Parser, Debug)]
#[command(name = "tanuki-mcp")]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, env = "TANUKI_MCP_CONFIG")]
    config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "TANUKI_MCP_LOG_LEVEL", default_value = "info")]
    log_level: String,

    /// Transport mode (stdio, http)
    #[arg(long, env = "TANUKI_MCP_TRANSPORT")]
    transport: Option<String>,

    /// HTTP server host (for http transport)
    #[arg(long, env = "TANUKI_MCP_HTTP_HOST", default_value = "127.0.0.1")]
    http_host: String,

    /// HTTP server port (for http transport)
    #[arg(long, env = "TANUKI_MCP_HTTP_PORT", default_value_t = DEFAULT_HTTP_PORT)]
    http_port: u16,

    /// Disable the dashboard
    #[arg(long, env = "TANUKI_MCP_NO_DASHBOARD")]
    no_dashboard: bool,

    /// Dashboard host
    #[arg(long, env = "TANUKI_MCP_DASHBOARD_HOST", default_value = "127.0.0.1")]
    dashboard_host: String,

    /// Dashboard port
    #[arg(long, env = "TANUKI_MCP_DASHBOARD_PORT", default_value_t = DEFAULT_DASHBOARD_PORT)]
    dashboard_port: u16,
}

fn create_handler(
    config: &AppConfig,
    gitlab: Arc<GitLabClient>,
    access: Arc<AccessResolver>,
) -> GitLabMcpHandler {
    GitLabMcpHandler::new_with_shared(config, gitlab, access)
}

fn create_handler_with_metrics(
    config: &AppConfig,
    gitlab: Arc<GitLabClient>,
    access: Arc<AccessResolver>,
    metrics: Arc<DashboardMetrics>,
) -> GitLabMcpHandler {
    GitLabMcpHandler::new_with_metrics(config, gitlab, access, metrics)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting GitLab MCP server"
    );

    // Load configuration
    let config = load_config(args.config.as_deref())
        .inspect_err(|e| error!(error = %e, "Failed to load configuration"))?;

    // Create auth provider
    let auth = create_auth_provider(&config.gitlab)
        .inspect_err(|e| error!(error = %e, "Failed to create auth provider"))?;

    // Create GitLab client
    let gitlab = Arc::new(
        GitLabClient::new(&config.gitlab, auth)
            .inspect_err(|e| error!(error = %e, "Failed to create GitLab client"))?,
    );

    // Create access control resolver
    let access = Arc::new(
        AccessResolver::new(&config.access_control)
            .inspect_err(|e| error!(error = %e, "Failed to create access resolver"))?,
    );

    // Create shared metrics collector
    let metrics = Arc::new(DashboardMetrics::new());

    // Determine if dashboard is enabled
    let dashboard_enabled = !args.no_dashboard && config.dashboard.enabled;

    // Start dashboard if enabled (in background)
    let dashboard_handle = if dashboard_enabled {
        let dashboard_config = DashboardConfig::new(&args.dashboard_host, args.dashboard_port)
            .unwrap_or_else(|_| {
                DashboardConfig::new(&config.dashboard.host, config.dashboard.port)
                    .unwrap_or_default()
            });

        let metrics_clone = metrics.clone();
        let app_config = Arc::new(config.clone());

        // Get tool count by creating a temporary handler
        let temp_handler = create_handler(&config, gitlab.clone(), access.clone());
        let tool_count = temp_handler.tool_count();

        Some(tokio::spawn(async move {
            if let Err(e) =
                run_dashboard(dashboard_config, metrics_clone, app_config, tool_count).await
            {
                error!(error = %e, "Dashboard server error");
            }
        }))
    } else {
        info!("Dashboard is disabled");
        None
    };

    // Determine transport mode
    let transport = args
        .transport
        .as_deref()
        .map(|t| match t {
            "stdio" => TransportMode::Stdio,
            "http" => TransportMode::Http,
            _ => config.server.transport,
        })
        .unwrap_or(config.server.transport);

    // Run the appropriate transport
    match transport {
        TransportMode::Stdio => {
            let handler = create_handler_with_metrics(&config, gitlab, access, metrics);
            run_stdio(handler).await?;
        }
        TransportMode::Http => {
            let http_config = HttpConfig::from_host_port(&args.http_host, args.http_port)?;

            // Clone the shared resources for the factory closure
            let config = Arc::new(config);

            run_http_blocking(
                move || {
                    create_handler_with_metrics(
                        &config,
                        gitlab.clone(),
                        access.clone(),
                        metrics.clone(),
                    )
                },
                http_config,
            )
            .await?;
        }
    }

    // Clean up dashboard if it was running
    if let Some(handle) = dashboard_handle {
        handle.abort();
    }

    Ok(())
}
