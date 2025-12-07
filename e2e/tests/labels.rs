//! E2E tests for label tools.
//!
//! Tests: list_labels, get_label, create_label, update_label, delete_label

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing labels.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_labels(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let label_name = common::unique_name("list-label");

    // Create a label first
    let _ = ctx
        .client
        .call_tool_json(
            "create_label",
            json!({
                "project": project_path,
                "name": label_name,
                "color": "#FF0000"
            }),
        )
        .await
        .expect("Failed to create label");

    let result = ctx
        .client
        .call_tool_json("list_labels", json!({ "project": project_path }))
        .await
        .expect("Failed to list labels");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let labels = result.as_array().unwrap();
    assert!(!labels.is_empty(), "Expected at least one label");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific label.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_label(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let label_name = common::unique_name("get-label");

    let _ = ctx
        .client
        .call_tool_json(
            "create_label",
            json!({
                "project": project_path,
                "name": label_name,
                "color": "#00FF00"
            }),
        )
        .await
        .expect("Failed to create label");

    let result = ctx
        .client
        .call_tool_json(
            "get_label",
            json!({
                "project": project_path,
                "label_id": label_name
            }),
        )
        .await
        .expect("Failed to get label");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(label_name.as_str())
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a label.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_label(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let label_name = common::unique_name("create-label");

    let result = ctx
        .client
        .call_tool_json(
            "create_label",
            json!({
                "project": project_path,
                "name": label_name,
                "color": "#0000FF",
                "description": "Test label description"
            }),
        )
        .await
        .expect("Failed to create label");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some(label_name.as_str())
    );
    assert_eq!(
        result.get("color").and_then(|v| v.as_str()),
        Some("#0000FF")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a label.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_label(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let label_name = common::unique_name("update-label");

    let _ = ctx
        .client
        .call_tool_json(
            "create_label",
            json!({
                "project": project_path,
                "name": label_name,
                "color": "#FFFF00"
            }),
        )
        .await
        .expect("Failed to create label");

    let result = ctx
        .client
        .call_tool_json(
            "update_label",
            json!({
                "project": project_path,
                "label_id": label_name,
                "new_name": "updated-label",
                "color": "#FF00FF",
                "description": "Updated description"
            }),
        )
        .await
        .expect("Failed to update label");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("updated-label")
    );
    assert_eq!(
        result.get("color").and_then(|v| v.as_str()),
        Some("#FF00FF")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a label.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_label(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let label_name = common::unique_name("delete-label");

    let _ = ctx
        .client
        .call_tool_json(
            "create_label",
            json!({
                "project": project_path,
                "name": label_name,
                "color": "#00FFFF"
            }),
        )
        .await
        .expect("Failed to create label");

    let result = ctx
        .client
        .call_tool(
            "delete_label",
            json!({
                "project": project_path,
                "label_id": label_name
            }),
        )
        .await
        .expect("Failed to delete label");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
