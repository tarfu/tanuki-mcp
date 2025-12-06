//! GitLab MCP Server
//!
//! A Model Context Protocol server for GitLab with fine-grained access control.
//!
//! ## Features
//!
//! - **95 GitLab tools** covering issues, merge requests, pipelines, repositories, and more
//! - **Hierarchical access control** with patterns at global, category, action, and project levels
//! - **Multiple transports** - stdio for Claude Code, HTTP/SSE for web integrations
//! - **Flexible configuration** via TOML files and environment variables
//!
//! ## Access Control Model
//!
//! ```text
//! all (base) → categories → individual actions → project overrides
//! ```
//!
//! Each level supports:
//! - Access levels: `none`, `read`, `full`
//! - `deny` patterns (regex) to block specific tools
//! - `allow` patterns (regex) to permit tools (overrides deny at same level)
//!
//! ## Example Configuration
//!
//! ```toml
//! [gitlab]
//! url = "https://gitlab.com"
//! # token from GITLAB_TOKEN env var
//!
//! [access_control]
//! all = "read"                    # Base: read-only
//! deny = ["delete_.*"]            # Block all deletes
//!
//! [access_control.categories.issues]
//! level = "full"                  # Full access to issues
//!
//! [access_control.projects."prod/app"]
//! all = "read"                    # Production is read-only
//! ```

pub mod access_control;
pub mod auth;
pub mod config;
pub mod dashboard;
pub mod error;
pub mod gitlab;
pub mod server;
pub mod tools;
pub mod transport;
pub mod util;

// Re-export main types
pub use config::{AppConfig, load_config};
pub use dashboard::{DashboardConfig, DashboardMetrics};
pub use error::{AppError, Result};
pub use server::GitLabMcpHandler;
