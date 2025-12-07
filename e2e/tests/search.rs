//! E2E tests for search tools.
//!
//! Tests: search_global, search_project, search_group

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test global search.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_search_global(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    // Wait for GitLab's search index to update
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "search_global",
            json!({
                "scope": "projects",
                "search": "test"
            }),
        )
        .await
        .expect("Failed to search globally");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test project search.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_search_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create a file with searchable content
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "searchable.txt",
                "branch": "main",
                "content": "This is searchable content for testing",
                "commit_message": "Add searchable file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Wait for GitLab's search index to update
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "search_project",
            json!({
                "project": project_path,
                "scope": "blobs",
                "search": "searchable"
            }),
        )
        .await
        .expect("Failed to search project");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test group search.
/// Note: This test requires at least one group to exist.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_search_group(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context") else { return; };

    // List groups first to get a valid group ID
    let groups = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    if let Some(group) = groups.as_array().and_then(|arr| arr.first()) {
        let group_id = group
            .get("id")
            .and_then(|v| v.as_i64())
            .expect("No group id");

        let result = ctx
            .client
            .call_tool_json(
                "search_group",
                json!({
                    "group_id": group_id,
                    "scope": "projects",
                    "search": "test"
                }),
            )
            .await
            .expect("Failed to search group");

        assert!(result.is_array(), "Expected array, got: {:?}", result);
    }
    // If no groups exist, test passes without assertion

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test global search for issues.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_search_global_issues(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create an issue to search for
    let _ = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Searchable Issue Title",
                "description": "This issue should be findable"
            }),
        )
        .await
        .expect("Failed to create issue");

    // Wait for search index
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "search_global",
            json!({
                "scope": "issues",
                "search": "Searchable"
            }),
        )
        .await
        .expect("Failed to search issues");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test project search for commits.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_search_project_commits(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Wait for search index
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "search_project",
            json!({
                "project": project_path,
                "scope": "commits",
                "search": "Initial"
            }),
        )
        .await
        .expect("Failed to search commits");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
