//! E2E tests for group tools.
//!
//! Tests: list_groups, get_group, list_group_members, list_group_projects, list_subgroups
//!
//! Note: Group creation requires admin permissions in GitLab CE.

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing groups.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_groups(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let result = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    // GitLab CE may not have any groups by default

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific group.
/// Note: This test requires at least one group to exist.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_group(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    // List groups first to get a valid group ID
    let groups = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    if let Some(group) = groups.as_array().and_then(|arr| arr.first()) {
        let group_id = group
            .get("id")
            .and_then(|v| v.as_i64())
            .expect("No group id");

        let result = ctx
            .client
            .call_tool_json("get_group", json!({ "group_id": group_id }))
            .await
            .expect("Failed to get group");

        assert!(result.is_object(), "Expected object, got: {:?}", result);
        assert_eq!(result.get("id").and_then(|v| v.as_i64()), Some(group_id));
    }
    // If no groups exist, test passes without assertion

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing group members.
/// Note: This test requires at least one group to exist.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_group_members(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let groups = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    if let Some(group) = groups.as_array().and_then(|arr| arr.first()) {
        let group_id = group
            .get("id")
            .and_then(|v| v.as_i64())
            .expect("No group id");

        let result = ctx
            .client
            .call_tool_json("list_group_members", json!({ "group_id": group_id }))
            .await
            .expect("Failed to list group members");

        assert!(result.is_array(), "Expected array, got: {:?}", result);
    }

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing group projects.
/// Note: This test requires at least one group to exist.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_group_projects(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let groups = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    if let Some(group) = groups.as_array().and_then(|arr| arr.first()) {
        let group_id = group
            .get("id")
            .and_then(|v| v.as_i64())
            .expect("No group id");

        let result = ctx
            .client
            .call_tool_json("list_group_projects", json!({ "group_id": group_id }))
            .await
            .expect("Failed to list group projects");

        assert!(result.is_array(), "Expected array, got: {:?}", result);
    }

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing subgroups.
/// Note: This test requires at least one group with subgroups to exist.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_subgroups(#[case] transport: TransportKind) {
    common::init_tracing();

    let ctx = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context");

    let groups = ctx
        .client
        .call_tool_json("list_groups", json!({}))
        .await
        .expect("Failed to list groups");

    if let Some(group) = groups.as_array().and_then(|arr| arr.first()) {
        let group_id = group
            .get("id")
            .and_then(|v| v.as_i64())
            .expect("No group id");

        let result = ctx
            .client
            .call_tool_json("list_subgroups", json!({ "group_id": group_id }))
            .await
            .expect("Failed to list subgroups");

        assert!(result.is_array(), "Expected array, got: {:?}", result);
    }

    ctx.cleanup().await.expect("Cleanup failed");
}
