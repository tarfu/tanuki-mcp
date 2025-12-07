//! E2E tests for user tools.
//!
//! Tests: get_current_user, list_users, get_user, get_user_activities

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test getting current user.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_current_user(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let result = ctx
        .client
        .call_tool_json("get_current_user", json!({}))
        .await
        .expect("Failed to get current user");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Expected id field");
    assert!(result.get("username").is_some(), "Expected username field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing users.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_users(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    let result = ctx
        .client
        .call_tool_json("list_users", json!({}))
        .await
        .expect("Failed to list users");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let users = result.as_array().unwrap();
    // GitLab CE should have at least the root user
    assert!(!users.is_empty(), "Expected at least one user");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific user.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_user(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    // Get current user first to get a valid user ID
    let current = ctx
        .client
        .call_tool_json("get_current_user", json!({}))
        .await
        .expect("Failed to get current user");

    let user_id = current
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No user id");

    let result = ctx
        .client
        .call_tool_json("get_user", json!({ "user_id": user_id }))
        .await
        .expect("Failed to get user");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_i64()), Some(user_id));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting user activities.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_user_activities(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .build()
        .await
        .expect("Failed to create context")
    else {
        return;
    };

    // Get current user to get a valid user_id
    let user = ctx
        .client
        .call_tool_json("get_current_user", json!({}))
        .await
        .expect("Failed to get current user");

    let user_id = user.get("id").and_then(|v| v.as_u64()).expect("No user id");

    let result = ctx
        .client
        .call_tool_json("get_user_activities", json!({ "user_id": user_id }))
        .await
        .expect("Failed to get user activities");

    // User activities returns an array
    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
