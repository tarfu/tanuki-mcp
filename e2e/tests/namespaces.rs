//! E2E tests for namespace tools.
//!
//! Tests: list_namespaces, get_namespace, namespace_exists

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing namespaces.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_namespaces(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let result = ctx
        .client
        .call_tool_json("list_namespaces", json!({}))
        .await
        .expect("Failed to list namespaces");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let namespaces = result.as_array().unwrap();
    // Should have at least the root user namespace
    assert!(!namespaces.is_empty(), "Expected at least one namespace");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific namespace.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_namespace(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    // List namespaces first to get a valid namespace ID
    let namespaces = ctx
        .client
        .call_tool_json("list_namespaces", json!({}))
        .await
        .expect("Failed to list namespaces");

    let namespace = namespaces
        .as_array()
        .and_then(|arr| arr.first())
        .expect("No namespaces found");

    let namespace_id = namespace
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No namespace id");

    let result = ctx
        .client
        .call_tool_json("get_namespace", json!({ "namespace_id": namespace_id }))
        .await
        .expect("Failed to get namespace");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("id").and_then(|v| v.as_i64()),
        Some(namespace_id)
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test checking if namespace exists.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_namespace_exists(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    // Get current user's namespace
    let user = ctx
        .client
        .call_tool_json("get_current_user", json!({}))
        .await
        .expect("Failed to get current user");

    let username = user
        .get("username")
        .and_then(|v| v.as_str())
        .expect("No username");

    // Check if user's namespace exists
    let result = ctx
        .client
        .call_tool_json("namespace_exists", json!({ "namespace": username }))
        .await
        .expect("Failed to check namespace exists");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("exists").is_some(), "Expected exists field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test namespace_exists with non-existent namespace.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_namespace_not_exists(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let non_existent = common::unique_name("nonexistent-namespace");

    let result = ctx
        .client
        .call_tool_json("namespace_exists", json!({ "namespace": non_existent }))
        .await
        .expect("Failed to check namespace exists");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    // The namespace should not exist
    let exists = result.get("exists").and_then(|v| v.as_bool());
    assert_eq!(exists, Some(false), "Expected namespace to not exist");

    ctx.cleanup().await.expect("Cleanup failed");
}
