//! E2E tests for issue note tools.
//!
//! Tests: list_issue_notes, get_issue_note, create_issue_note, update_issue_note, delete_issue_note

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Helper to create an issue and return its IID.
async fn create_test_issue(ctx: &tanuki_mcp_e2e::TestContext, project: &str) -> u64 {
    let result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project,
                "title": common::unique_name("test-issue")
            }),
        )
        .await
        .expect("Failed to create issue");

    result
        .get("iid")
        .and_then(|v| v.as_u64())
        .expect("No issue IID")
}

/// Test listing issue notes.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_issue_notes(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path).await;

    let result = ctx
        .client
        .call_tool_json(
            "list_issue_notes",
            json!({
                "project": project_path,
                "issue_iid": issue_iid
            }),
        )
        .await
        .expect("Failed to list issue notes");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating an issue note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_issue_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path).await;

    let note_body = "This is a test note from E2E tests";
    let result = ctx
        .client
        .call_tool_json(
            "create_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "body": note_body
            }),
        )
        .await
        .expect("Failed to create issue note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Missing note ID");
    assert_eq!(result.get("body").and_then(|v| v.as_str()), Some(note_body));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific issue note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_issue_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path).await;

    // Create a note first
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "body": "Note to get"
            }),
        )
        .await
        .expect("Failed to create issue note");

    let note_id = create_result
        .get("id")
        .and_then(|v| v.as_u64())
        .expect("No note ID");

    // Get the note
    let result = ctx
        .client
        .call_tool_json(
            "get_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "note_id": note_id
            }),
        )
        .await
        .expect("Failed to get issue note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_u64()), Some(note_id));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating an issue note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_issue_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path).await;

    // Create a note first
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "body": "Original note"
            }),
        )
        .await
        .expect("Failed to create issue note");

    let note_id = create_result
        .get("id")
        .and_then(|v| v.as_u64())
        .expect("No note ID");

    // Update the note
    let new_body = "Updated note body";
    let result = ctx
        .client
        .call_tool_json(
            "update_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "note_id": note_id,
                "body": new_body
            }),
        )
        .await
        .expect("Failed to update issue note");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("body").and_then(|v| v.as_str()), Some(new_body));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting an issue note.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_issue_note(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path).await;

    // Create a note to delete
    let create_result = ctx
        .client
        .call_tool_json(
            "create_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "body": "Note to delete"
            }),
        )
        .await
        .expect("Failed to create issue note");

    let note_id = create_result
        .get("id")
        .and_then(|v| v.as_u64())
        .expect("No note ID");

    // Delete the note
    let result = ctx
        .client
        .call_tool(
            "delete_issue_note",
            json!({
                "project": project_path,
                "issue_iid": issue_iid,
                "note_id": note_id
            }),
        )
        .await
        .expect("Failed to delete issue note");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
