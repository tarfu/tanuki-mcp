//! E2E tests for merge request tools.
//!
//! Tests: list_merge_requests, get_merge_request, create_merge_request,
//!        update_merge_request, merge_merge_request, get_merge_request_diffs

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Helper to create a merge request for testing.
async fn create_test_mr(
    ctx: &tanuki_mcp_e2e::TestContext,
    project_path: &str,
    source_branch: &str,
) -> serde_json::Value {
    // Create source branch
    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": source_branch,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create source branch");

    // Add a file to make the branches diverge
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": format!("{}-file.txt", source_branch),
                "branch": source_branch,
                "content": "MR test content",
                "commit_message": "Add MR test file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Create MR
    ctx.client
        .call_tool_json(
            "create_merge_request",
            json!({
                "project": project_path,
                "source_branch": source_branch,
                "target_branch": "main",
                "title": format!("Test MR from {}", source_branch)
            }),
        )
        .await
        .expect("Failed to create MR")
}

/// Test listing merge requests.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_merge_requests(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("list-mr-branch");

    // Create an MR
    let _ = create_test_mr(&ctx, &project_path, &branch_name).await;

    let result = ctx
        .client
        .call_tool_json("list_merge_requests", json!({ "project": project_path }))
        .await
        .expect("Failed to list merge requests");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let mrs = result.as_array().unwrap();
    assert!(!mrs.is_empty(), "Expected at least one MR");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific merge request.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_merge_request(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("get-mr-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    let result = ctx
        .client
        .call_tool_json(
            "get_merge_request",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to get merge request");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("iid").and_then(|v| v.as_i64()), Some(mr_iid));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a merge request.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_merge_request(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("create-mr-branch");

    // Create a branch with changes
    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": branch_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create branch");

    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "mr-test.txt",
                "branch": branch_name,
                "content": "MR content",
                "commit_message": "Add MR file"
            }),
        )
        .await
        .expect("Failed to create file");

    let result = ctx
        .client
        .call_tool_json(
            "create_merge_request",
            json!({
                "project": project_path,
                "source_branch": branch_name,
                "target_branch": "main",
                "title": "Test MR creation",
                "description": "This is a test MR"
            }),
        )
        .await
        .expect("Failed to create merge request");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some("Test MR creation")
    );
    assert!(result.get("iid").is_some(), "Expected iid field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a merge request.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_merge_request(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("update-mr-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    let result = ctx
        .client
        .call_tool_json(
            "update_merge_request",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "title": "Updated MR title",
                "description": "Updated description"
            }),
        )
        .await
        .expect("Failed to update merge request");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some("Updated MR title")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test merging a merge request.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_merge_merge_request(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("merge-mr-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Wait a moment for GitLab to process the MR
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "merge_merge_request",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to merge MR");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    // After merging, state should be "merged"
    let state = result.get("state").and_then(|v| v.as_str());
    assert!(
        state == Some("merged") || state == Some("can_be_merged"),
        "Expected merged state, got: {:?}",
        state
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting merge request diffs.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_merge_request_diffs(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("diffs-mr-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    let result = ctx
        .client
        .call_tool_json(
            "get_merge_request_diffs",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to get MR diffs");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let diffs = result.as_array().unwrap();
    assert!(!diffs.is_empty(), "Expected at least one diff");

    ctx.cleanup().await.expect("Cleanup failed");
}
