//! E2E tests for issue tools.
//!
//! Tests: list_issues, get_issue, create_issue, update_issue, delete_issue

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing issues in a project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_issues(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json("list_issues", json!({ "project": project_path }))
        .await
        .expect("Failed to list issues");

    // Result should be an array (may be empty initially)
    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating an issue.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_issue(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_title = common::unique_name("test-issue");

    let result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": issue_title,
                "description": "E2E test issue description"
            }),
        )
        .await
        .expect("Failed to create issue");

    // Verify issue was created
    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("iid").is_some(), "Missing issue IID");
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some(issue_title.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific issue.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_issue(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");

    // First create an issue
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Test issue for get"
            }),
        )
        .await
        .expect("Failed to create issue");

    let issue_iid = create_result
        .get("iid")
        .and_then(|v| v.as_u64())
        .expect("No issue IID");

    // Now get the issue
    let result = ctx
        .client
        .call_tool_json(
            "get_issue",
            json!({
                "project": project_path,
                "issue_iid": issue_iid
            }),
        )
        .await
        .expect("Failed to get issue");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("iid").and_then(|v| v.as_u64()), Some(issue_iid));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating an issue.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_issue(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");

    // First create an issue
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Original title"
            }),
        )
        .await
        .expect("Failed to create issue");

    let issue_iid = create_result
        .get("iid")
        .and_then(|v| v.as_u64())
        .expect("No issue IID");

    // Update the issue
    let new_title = "Updated title";
    let result = ctx
        .client
        .call_tool_json(
            "update_issue",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "title": new_title,
                "description": "Updated description"
            }),
        )
        .await
        .expect("Failed to update issue");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some(new_title)
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting an issue.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_issue(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");

    // First create an issue to delete
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Issue to delete"
            }),
        )
        .await
        .expect("Failed to create issue");

    let issue_iid = create_result
        .get("iid")
        .and_then(|v| v.as_u64())
        .expect("No issue IID");

    // Delete the issue
    let result = ctx
        .client
        .call_tool(
            "delete_issue",
            json!({
                "project": project_path,
                "issue_iid": issue_iid
            }),
        )
        .await
        .expect("Failed to delete issue");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing issues with filters.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_issues_with_filters(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create an issue first
    let _ = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Searchable issue"
            }),
        )
        .await
        .expect("Failed to create issue");

    // List with state filter
    let result = ctx
        .client
        .call_tool_json(
            "list_issues",
            json!({
                "project": project_path,
                "state": "opened"
            }),
        )
        .await
        .expect("Failed to list issues");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let issues = result.as_array().unwrap();
    assert!(!issues.is_empty(), "Expected at least one open issue");

    ctx.cleanup().await.expect("Cleanup failed");
}
