//! E2E tests for milestone tools.
//!
//! Tests: list_milestones, get_milestone, create_milestone, update_milestone,
//!        delete_milestone, get_milestone_issues, get_milestone_merge_requests

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing milestones.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_milestones(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("list-milestone");

    // Create a milestone first
    let _ = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let result = ctx
        .client
        .call_tool_json("list_milestones", json!({ "project": project_path }))
        .await
        .expect("Failed to list milestones");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let milestones = result.as_array().unwrap();
    assert!(!milestones.is_empty(), "Expected at least one milestone");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific milestone.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_milestone(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("get-milestone");

    let milestone = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let milestone_id = milestone
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No milestone id");

    let result = ctx
        .client
        .call_tool_json(
            "get_milestone",
            json!({
                "project": project_path,
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to get milestone");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("id").and_then(|v| v.as_i64()),
        Some(milestone_id)
    );
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some(milestone_title.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a milestone.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_milestone(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("create-milestone");

    let result = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title,
                "description": "Test milestone description"
            }),
        )
        .await
        .expect("Failed to create milestone");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some(milestone_title.as_str())
    );
    assert!(result.get("id").is_some(), "Expected id field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a milestone.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_milestone(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("update-milestone");

    let milestone = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let milestone_id = milestone
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No milestone id");

    let result = ctx
        .client
        .call_tool_json(
            "update_milestone",
            json!({
                "project": project_path,
                "milestone_id": milestone_id,
                "title": "Updated Milestone Title",
                "description": "Updated description"
            }),
        )
        .await
        .expect("Failed to update milestone");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some("Updated Milestone Title")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a milestone.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_milestone(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("delete-milestone");

    let milestone = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let milestone_id = milestone
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No milestone id");

    let result = ctx
        .client
        .call_tool(
            "delete_milestone",
            json!({
                "project": project_path,
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to delete milestone");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting milestone issues.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_milestone_issues(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("issues-milestone");

    // Create a milestone
    let milestone = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let milestone_id = milestone
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No milestone id");

    // Create an issue with the milestone
    let _ = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project_path,
                "title": "Issue for milestone",
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to create issue");

    let result = ctx
        .client
        .call_tool_json(
            "get_milestone_issues",
            json!({
                "project": project_path,
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to get milestone issues");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let issues = result.as_array().unwrap();
    assert!(!issues.is_empty(), "Expected at least one issue");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting milestone merge requests.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_milestone_merge_requests(#[case] transport: TransportKind) {
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
    let milestone_title = common::unique_name("mr-milestone");
    let branch_name = common::unique_name("milestone-mr-branch");

    // Create a milestone
    let milestone = ctx
        .client
        .call_tool_json(
            "create_milestone",
            json!({
                "project": project_path,
                "title": milestone_title
            }),
        )
        .await
        .expect("Failed to create milestone");

    let milestone_id = milestone
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No milestone id");

    // Create a branch and MR with the milestone
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
                "file_path": "milestone-test.txt",
                "branch": branch_name,
                "content": "Milestone MR content",
                "commit_message": "Add milestone file"
            }),
        )
        .await
        .expect("Failed to create file");

    let _ = ctx
        .client
        .call_tool_json(
            "create_merge_request",
            json!({
                "project": project_path,
                "source_branch": branch_name,
                "target_branch": "main",
                "title": "MR for milestone",
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to create MR");

    let result = ctx
        .client
        .call_tool_json(
            "get_milestone_merge_requests",
            json!({
                "project": project_path,
                "milestone_id": milestone_id
            }),
        )
        .await
        .expect("Failed to get milestone merge requests");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let mrs = result.as_array().unwrap();
    assert!(!mrs.is_empty(), "Expected at least one merge request");

    ctx.cleanup().await.expect("Cleanup failed");
}
