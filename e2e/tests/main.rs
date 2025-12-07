//! Single E2E test binary - all tests share one MCP server instance.
//!
//! By consolidating all tests into a single binary, the `OnceLock` singleton
//! in `shared.rs` ensures only one stdio server is spawned for all tests.
//!
//! # Test Templates
//!
//! ## Basic test (no project needed)
//!
//! ```rust,ignore
//! #[rstest]
//! #[case::stdio(TransportKind::Stdio)]
//! #[case::http(TransportKind::Http)]
//! #[tokio::test]
//! async fn test_example(#[case] transport: TransportKind) {
//!     common::init_tracing();
//!
//!     let Some(ctx) = TestContext::new(transport)
//!         .await
//!         .expect("Failed to create context")
//!     else {
//!         return;
//!     };
//!
//!     // Your test logic here
//!     let result = ctx.client.call_tool_json("tool_name", json!({}))
//!         .await
//!         .expect("Failed");
//!     assert!(result.is_array());
//!
//!     ctx.cleanup().await.expect("Cleanup failed");
//! }
//! ```
//!
//! ## Test with project
//!
//! ```rust,ignore
//! #[rstest]
//! #[case::stdio(TransportKind::Stdio)]
//! #[case::http(TransportKind::Http)]
//! #[tokio::test]
//! async fn test_with_project(#[case] transport: TransportKind) {
//!     common::init_tracing();
//!
//!     let Some(ctx) = TestContextBuilder::new(transport)
//!         .with_project()
//!         .build()
//!         .await
//!         .expect("Failed to create context")
//!     else {
//!         return;
//!     };
//!
//!     let project_path = ctx.project_path.clone().expect("No project path");
//!
//!     // Your test logic here
//!     let result = ctx.client.call_tool_json("tool_name", json!({ "project": project_path }))
//!         .await
//!         .expect("Failed");
//!     assert!(result.is_object());
//!
//!     ctx.cleanup().await.expect("Cleanup failed");
//! }
//! ```

mod common;

mod branches;
mod commits;
mod groups;
mod issue_links;
mod issue_notes;
mod issues;
mod jobs;
mod labels;
mod merge_requests;
mod milestones;
mod mr_discussions;
mod mr_drafts;
mod namespaces;
mod pipelines;
mod projects;
mod releases;
mod repository;
mod search;
mod tags;
mod users;
mod wiki;
