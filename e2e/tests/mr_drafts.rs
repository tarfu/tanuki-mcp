//! E2E tests for merge request draft note tools.
//!
//! Tests: list_mr_draft_notes, get_mr_draft_note, create_mr_draft_note,
//!        update_mr_draft_note, delete_mr_draft_note,
//!        publish_mr_draft_note, publish_all_mr_draft_notes

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
                "content": "Draft note test content",
                "commit_message": "Add draft note test file"
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

/// Test listing MR draft notes.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_mr_draft_notes(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("list-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a draft note first
    let _ = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Test draft note for listing"
            }),
        )
        .await
        .expect("Failed to create draft note");

    let result = ctx
        .client
        .call_tool_json(
            "list_mr_draft_notes",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to list draft notes");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let drafts = result.as_array().unwrap();
    assert!(!drafts.is_empty(), "Expected at least one draft note");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific MR draft note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_mr_draft_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("get-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a draft note
    let draft = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Test draft note to get"
            }),
        )
        .await
        .expect("Failed to create draft note");

    let draft_id = draft
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No draft id");

    let result = ctx
        .client
        .call_tool_json(
            "get_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "draft_note_id": draft_id
            }),
        )
        .await
        .expect("Failed to get draft note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_i64()), Some(draft_id));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating an MR draft note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_mr_draft_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("create-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    let result = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "This is a test draft note"
            }),
        )
        .await
        .expect("Failed to create draft note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Expected id field");
    assert_eq!(
        result.get("note").and_then(|v| v.as_str()),
        Some("This is a test draft note")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating an MR draft note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_mr_draft_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("update-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a draft note
    let draft = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Draft note to update"
            }),
        )
        .await
        .expect("Failed to create draft note");

    let draft_id = draft
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No draft id");

    let result = ctx
        .client
        .call_tool_json(
            "update_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "draft_note_id": draft_id,
                "note": "Updated draft note content"
            }),
        )
        .await
        .expect("Failed to update draft note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("note").and_then(|v| v.as_str()),
        Some("Updated draft note content")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting an MR draft note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_mr_draft_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("delete-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a draft note to delete
    let draft = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Draft note to delete"
            }),
        )
        .await
        .expect("Failed to create draft note");

    let draft_id = draft
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No draft id");

    let result = ctx
        .client
        .call_tool(
            "delete_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "draft_note_id": draft_id
            }),
        )
        .await
        .expect("Failed to delete draft note");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test publishing an MR draft note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_publish_mr_draft_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("publish-draft-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a draft note to publish
    let draft = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Draft note to publish"
            }),
        )
        .await
        .expect("Failed to create draft note");

    let draft_id = draft
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No draft id");

    let result = ctx
        .client
        .call_tool(
            "publish_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "draft_note_id": draft_id
            }),
        )
        .await
        .expect("Failed to publish draft note");

    assert!(
        result.is_error != Some(true),
        "Publish returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test publishing all MR draft notes.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_publish_all_mr_draft_notes(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("publish-all-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create multiple draft notes
    let _ = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Draft note 1"
            }),
        )
        .await
        .expect("Failed to create draft note 1");

    let _ = ctx
        .client
        .call_tool_json(
            "create_mr_draft_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "note": "Draft note 2"
            }),
        )
        .await
        .expect("Failed to create draft note 2");

    let result = ctx
        .client
        .call_tool(
            "publish_all_mr_draft_notes",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to publish all draft notes");

    assert!(
        result.is_error != Some(true),
        "Publish all returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
