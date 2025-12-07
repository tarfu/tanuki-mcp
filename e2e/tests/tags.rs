//! E2E tests for tag tools.
//!
//! Tests: list_tags, get_tag, create_tag, delete_tag,
//!        list_protected_tags, get_protected_tag, protect_tag, unprotect_tag

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing tags.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_tags(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create a tag first so we have something to list
    let tag_name = common::unique_name("list-tag");
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    let result = ctx
        .client
        .call_tool_json("list_tags", json!({ "project": project_path }))
        .await
        .expect("Failed to list tags");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let tags = result.as_array().unwrap();
    assert!(!tags.is_empty(), "Expected at least one tag");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("get-tag");

    // Create a tag to get
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    let result = ctx
        .client
        .call_tool_json(
            "get_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name
            }),
        )
        .await
        .expect("Failed to get tag");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("create-tag");

    let result = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main",
                "message": "Test tag message"
            }),
        )
        .await
        .expect("Failed to create tag");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("delete-tag");

    // Create a tag to delete
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    // Delete the tag
    let result = ctx
        .client
        .call_tool(
            "delete_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name
            }),
        )
        .await
        .expect("Failed to delete tag");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing protected tags.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_protected_tags(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create and protect a tag first
    let tag_name = common::unique_name("protected-tag");
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    let _ = ctx
        .client
        .call_tool_json(
            "protect_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to protect tag");

    let result = ctx
        .client
        .call_tool_json("list_protected_tags", json!({ "project": project_path }))
        .await
        .expect("Failed to list protected tags");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let protected = result.as_array().unwrap();
    assert!(!protected.is_empty(), "Expected at least one protected tag");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a protected tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_protected_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("get-protected-tag");

    // Create and protect a tag
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    let _ = ctx
        .client
        .call_tool_json(
            "protect_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to protect tag");

    let result = ctx
        .client
        .call_tool_json(
            "get_protected_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to get protected tag");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test protecting a tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_protect_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("protect-tag");

    // Create a tag to protect
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    // Protect the tag
    let result = ctx
        .client
        .call_tool_json(
            "protect_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to protect tag");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test unprotecting a tag.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_unprotect_tag(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context");

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("unprotect-tag");

    // Create and protect a tag
    let _ = ctx
        .client
        .call_tool_json(
            "create_tag",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create tag");

    let _ = ctx
        .client
        .call_tool_json(
            "protect_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to protect tag");

    // Unprotect the tag
    let result = ctx
        .client
        .call_tool(
            "unprotect_tag",
            json!({
                "project": project_path,
                "name": tag_name
            }),
        )
        .await
        .expect("Failed to unprotect tag");

    assert!(
        result.is_error != Some(true),
        "Unprotect returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
