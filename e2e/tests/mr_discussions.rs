//! E2E tests for merge request discussion tools.
//!
//! Tests: list_mr_discussions, get_mr_discussion, create_mr_discussion,
//!        add_mr_discussion_note, update_mr_discussion_note,
//!        delete_mr_discussion_note, resolve_mr_discussion

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
                "content": "Discussion test content",
                "commit_message": "Add discussion test file"
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

/// Test listing MR discussions.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_mr_discussions(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("list-disc-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a discussion first
    let _ = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Test discussion for listing"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let result = ctx
        .client
        .call_tool_json(
            "list_mr_discussions",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid
            }),
        )
        .await
        .expect("Failed to list discussions");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let discussions = result.as_array().unwrap();
    assert!(!discussions.is_empty(), "Expected at least one discussion");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific MR discussion.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_mr_discussion(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("get-disc-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a discussion
    let discussion = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Test discussion to get"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let discussion_id = discussion
        .get("id")
        .and_then(|v| v.as_str())
        .expect("No discussion id");

    let result = ctx
        .client
        .call_tool_json(
            "get_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id
            }),
        )
        .await
        .expect("Failed to get discussion");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("id").and_then(|v| v.as_str()),
        Some(discussion_id)
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating an MR discussion.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_mr_discussion(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("create-disc-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    let result = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "This is a test discussion"
            }),
        )
        .await
        .expect("Failed to create discussion");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Expected id field");
    assert!(result.get("notes").is_some(), "Expected notes field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test adding a note to an MR discussion.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_add_mr_discussion_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("add-note-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a discussion
    let discussion = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Initial discussion"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let discussion_id = discussion
        .get("id")
        .and_then(|v| v.as_str())
        .expect("No discussion id");

    let result = ctx
        .client
        .call_tool_json(
            "add_mr_discussion_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id,
                "body": "Reply to discussion"
            }),
        )
        .await
        .expect("Failed to add note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Expected id field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating an MR discussion note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_mr_discussion_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("update-note-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a discussion
    let discussion = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Discussion to update"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let discussion_id = discussion
        .get("id")
        .and_then(|v| v.as_str())
        .expect("No discussion id");

    // Get the note ID from the first note in the discussion
    let note_id = discussion
        .get("notes")
        .and_then(|n| n.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("id"))
        .and_then(|v| v.as_i64())
        .expect("No note id");

    let result = ctx
        .client
        .call_tool_json(
            "update_mr_discussion_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id,
                "note_id": note_id,
                "body": "Updated discussion content"
            }),
        )
        .await
        .expect("Failed to update note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("body").and_then(|v| v.as_str()),
        Some("Updated discussion content")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting an MR discussion note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_mr_discussion_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("delete-note-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a discussion
    let discussion = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Discussion with note to delete"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let discussion_id = discussion
        .get("id")
        .and_then(|v| v.as_str())
        .expect("No discussion id");

    // Add a note to delete (can't delete the first note)
    let note = ctx
        .client
        .call_tool_json(
            "add_mr_discussion_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id,
                "body": "Note to be deleted"
            }),
        )
        .await
        .expect("Failed to add note");

    let note_id = note.get("id").and_then(|v| v.as_i64()).expect("No note id");

    let result = ctx
        .client
        .call_tool(
            "delete_mr_discussion_note",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id,
                "note_id": note_id
            }),
        )
        .await
        .expect("Failed to delete note");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test resolving an MR discussion.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_resolve_mr_discussion(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let branch_name = common::unique_name("resolve-disc-branch");

    let mr = create_test_mr(&ctx, &project_path, &branch_name).await;
    let mr_iid = mr.get("iid").and_then(|v| v.as_i64()).expect("No MR iid");

    // Create a resolvable discussion
    let discussion = ctx
        .client
        .call_tool_json(
            "create_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "body": "Discussion to resolve"
            }),
        )
        .await
        .expect("Failed to create discussion");

    let discussion_id = discussion
        .get("id")
        .and_then(|v| v.as_str())
        .expect("No discussion id");

    let result = ctx
        .client
        .call_tool_json(
            "resolve_mr_discussion",
            json!({
                "project": project_path,
                "merge_request_iid": mr_iid,
                "discussion_id": discussion_id,
                "resolved": true
            }),
        )
        .await
        .expect("Failed to resolve discussion");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
