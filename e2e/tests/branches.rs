//! E2E tests for branch tools.
//!
//! Tests: list_branches, get_branch, create_branch, delete_branch,
//!        protect_branch, unprotect_branch

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing branches.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_branches(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json("list_branches", json!({ "project": project_path }))
        .await
        .expect("Failed to list branches");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let branches = result.as_array().unwrap();
    // Project should have at least main/master branch
    assert!(!branches.is_empty(), "Expected at least one branch");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific branch.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_branch(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json(
            "get_branch",
            json!({
                "project": project_path,
                "branch": "main"
            }),
        )
        .await
        .expect("Failed to get branch");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("name").and_then(|v| v.as_str()), Some("main"));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a branch.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_branch(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("test-branch");

    let result = ctx
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

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(branch_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a branch.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_branch(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("delete-branch");

    // Create a branch to delete
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

    // Delete the branch
    let result = ctx
        .client
        .call_tool(
            "delete_branch",
            json!({
                "project": project_path,
                "branch": branch_name
            }),
        )
        .await
        .expect("Failed to delete branch");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test protecting a branch.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_protect_branch(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("protect-branch");

    // Create a branch to protect
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

    // Protect the branch
    let result = ctx
        .client
        .call_tool_json(
            "protect_branch",
            json!({
                "project": project_path,
                "name": branch_name
            }),
        )
        .await
        .expect("Failed to protect branch");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("name").is_some());

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test unprotecting a branch.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_unprotect_branch(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("unprotect-branch");

    // Create and protect a branch
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
            "protect_branch",
            json!({
                "project": project_path,
                "name": branch_name
            }),
        )
        .await
        .expect("Failed to protect branch");

    // Unprotect the branch
    let result = ctx
        .client
        .call_tool(
            "unprotect_branch",
            json!({
                "project": project_path,
                "name": branch_name
            }),
        )
        .await
        .expect("Failed to unprotect branch");

    assert!(
        result.is_error != Some(true),
        "Unprotect returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
