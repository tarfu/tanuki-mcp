//! End-to-end test framework for tanuki-mcp.
//!
//! This crate provides utilities for running E2E tests against a real GitLab CE
//! instance using Docker. Tests can run against both stdio and HTTP/SSE transports.
//!
//! # Architecture
//!
//! - `shared`: Shared MCP servers (one per transport) for efficient test execution
//! - `transport`: MCP client abstraction supporting stdio and HTTP transports
//! - `gitlab`: GitLab CE container management and API helpers
//! - `context`: Test context combining MCP client, GitLab access, and test resources
//!
//! # Usage
//!
//! ```rust,ignore
//! use tanuki_mcp_e2e::{TestContext, TransportKind};
//! use rstest::rstest;
//!
//! #[rstest]
//! #[case::stdio(TransportKind::Stdio)]
//! #[case::http(TransportKind::Http)]
//! #[tokio::test]
//! async fn test_list_projects(#[case] transport: TransportKind) {
//!     let ctx = TestContext::new(transport).await.unwrap();
//!     let result = ctx.client.call_tool_json("list_projects", json!({})).await.unwrap();
//!     assert!(result.is_array());
//!     ctx.cleanup().await.unwrap();
//! }
//! ```

pub mod context;
pub mod gitlab;
pub mod shared;
pub mod transport;

// Re-export main types for convenience
pub use context::{TestContext, TestContextBuilder};
pub use gitlab::{DEFAULT_GITLAB_PORT, DEFAULT_ROOT_PASSWORD, GitLabConfig, GitLabContainer};
pub use shared::{SharedMcpClient, SharedPeer, SharedServers, get_shared_servers};
pub use transport::{McpClient, TransportKind};
