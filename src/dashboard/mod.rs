//! Dashboard module
//!
//! Provides a web-based dashboard for monitoring the GitLab MCP server,
//! showing configuration, project access statistics, and tool usage metrics.

pub mod metrics;
pub mod server;

pub use metrics::{DashboardMetrics, ProjectStats, ToolStats};
pub use server::{run_dashboard, DashboardConfig, DEFAULT_DASHBOARD_PORT};
