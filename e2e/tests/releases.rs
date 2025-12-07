//! E2E tests for release tools.
//!
//! Tests: list_releases, get_release, create_release, update_release,
//!        delete_release, get_release_evidence

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing releases.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_releases(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("release-tag");

    // Create a tag first
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

    // Create a release
    let _ = ctx
        .client
        .call_tool_json(
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Test Release",
                "description": "Test release description"
            }),
        )
        .await
        .expect("Failed to create release");

    let result = ctx
        .client
        .call_tool_json("list_releases", json!({ "project": project_path }))
        .await
        .expect("Failed to list releases");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let releases = result.as_array().unwrap();
    assert!(!releases.is_empty(), "Expected at least one release");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific release.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_release(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("get-release-tag");

    // Create a tag and release
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
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Get Test Release"
            }),
        )
        .await
        .expect("Failed to create release");

    let result = ctx
        .client
        .call_tool_json(
            "get_release",
            json!({
                "project": project_path,
                "tag_name": tag_name
            }),
        )
        .await
        .expect("Failed to get release");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("tag_name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a release.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_release(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("create-release-tag");

    // Create a tag first
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
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "New Test Release",
                "description": "Description for new release"
            }),
        )
        .await
        .expect("Failed to create release");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("New Test Release")
    );
    assert_eq!(
        result.get("tag_name").and_then(|v| v.as_str()),
        Some(tag_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a release.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_release(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("update-release-tag");

    // Create a tag and release
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
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Original Release"
            }),
        )
        .await
        .expect("Failed to create release");

    let result = ctx
        .client
        .call_tool_json(
            "update_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Updated Release Name",
                "description": "Updated description"
            }),
        )
        .await
        .expect("Failed to update release");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("Updated Release Name")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a release.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_release(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("delete-release-tag");

    // Create a tag and release
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
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Release to Delete"
            }),
        )
        .await
        .expect("Failed to create release");

    let result = ctx
        .client
        .call_tool(
            "delete_release",
            json!({
                "project": project_path,
                "tag_name": tag_name
            }),
        )
        .await
        .expect("Failed to delete release");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting release evidence.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_release_evidence(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let tag_name = common::unique_name("evidence-release-tag");

    // Create a tag and release
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
            "create_release",
            json!({
                "project": project_path,
                "tag_name": tag_name,
                "name": "Evidence Release"
            }),
        )
        .await
        .expect("Failed to create release");

    // Note: Evidence collection happens asynchronously, so this may return empty
    let result = ctx
        .client
        .call_tool_json(
            "get_release_evidence",
            json!({
                "project": project_path,
                "tag_name": tag_name
            }),
        )
        .await
        .expect("Failed to get release evidence");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
