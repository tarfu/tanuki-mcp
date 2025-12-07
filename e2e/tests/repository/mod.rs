//! E2E tests for repository file tools.
//!
//! Tests: get_repository_tree, get_repository_file, create_or_update_file,
//!        delete_repository_file, get_file_blame

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test getting repository tree.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_repository_tree(#[case] transport: TransportKind) {
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
        .call_tool_json(
            "get_repository_tree",
            json!({
                "project": project_path
            }),
        )
        .await
        .expect("Failed to get repository tree");

    // Result should be an array (project was initialized with README)
    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a file.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_file(#[case] transport: TransportKind) {
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
    let file_path = format!("test-{}.txt", common::unique_name("file"));
    let content = "Hello from E2E test!";

    let result = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": content,
                "branch": "main",
                "commit_message": "Add test file via E2E"
            }),
        )
        .await
        .expect("Failed to create file");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("file_path").is_some() || result.get("file_name").is_some());

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a file.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_repository_file(#[case] transport: TransportKind) {
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

    // Create a file first
    let file_path = format!("test-{}.txt", common::unique_name("get"));
    let content = "Content to get";

    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": content,
                "branch": "main",
                "commit_message": "Add file for get test"
            }),
        )
        .await
        .expect("Failed to create file");

    // Now get the file
    let result = ctx
        .client
        .call_tool_json(
            "get_repository_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to get file");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("content").is_some() || result.get("file_name").is_some());

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a file.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_file(#[case] transport: TransportKind) {
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
    let file_path = format!("test-{}.txt", common::unique_name("update"));

    // Create a file first
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": "Original content",
                "branch": "main",
                "commit_message": "Add file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Update the file
    let new_content = "Updated content";
    let result = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": new_content,
                "branch": "main",
                "commit_message": "Update file via E2E"
            }),
        )
        .await
        .expect("Failed to update file");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a file.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_repository_file(#[case] transport: TransportKind) {
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
    let file_path = format!("test-{}.txt", common::unique_name("delete"));

    // Create a file to delete
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": "File to delete",
                "branch": "main",
                "commit_message": "Add file to delete"
            }),
        )
        .await
        .expect("Failed to create file");

    // Delete the file
    let result = ctx
        .client
        .call_tool(
            "delete_repository_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "branch": "main",
                "commit_message": "Delete file via E2E"
            }),
        )
        .await
        .expect("Failed to delete file");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting file blame.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_file_blame(#[case] transport: TransportKind) {
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
    let file_path = format!("test-{}.txt", common::unique_name("blame"));

    // Create a file first
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": file_path,
                "content": "Line 1\nLine 2\nLine 3",
                "branch": "main",
                "commit_message": "Add file for blame"
            }),
        )
        .await
        .expect("Failed to create file");

    // Get file blame
    let result = ctx
        .client
        .call_tool_json(
            "get_file_blame",
            json!({
                "project": project_path,
                "file_path": file_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to get file blame");

    assert!(
        result.is_array(),
        "Expected array (blame ranges), got: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting repository tree with path filter.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_repository_tree_with_path(#[case] transport: TransportKind) {
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

    // Create a file in a subdirectory
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "subdir/test.txt",
                "content": "Nested file",
                "branch": "main",
                "commit_message": "Add nested file"
            }),
        )
        .await
        .expect("Failed to create nested file");

    // Get tree for subdirectory
    let result = ctx
        .client
        .call_tool_json(
            "get_repository_tree",
            json!({
                "project": project_path,
                "path": "subdir"
            }),
        )
        .await
        .expect("Failed to get repository tree");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let tree = result.as_array().unwrap();
    assert!(!tree.is_empty(), "Expected files in subdir");

    ctx.cleanup().await.expect("Cleanup failed");
}
