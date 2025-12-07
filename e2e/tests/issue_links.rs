//! E2E tests for issue link tools.
//!
//! Tests: list_issue_links, create_issue_link, delete_issue_link

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Helper to create an issue and return its IID.
async fn create_test_issue(ctx: &tanuki_mcp_e2e::TestContext, project: &str, title: &str) -> u64 {
    let result = ctx
        .client
        .call_tool_json(
            "create_issue",
            json!({
                "project": project,
                "title": title
            }),
        )
        .await
        .expect("Failed to create issue");

    result
        .get("iid")
        .and_then(|v| v.as_u64())
        .expect("No issue IID")
}

/// Test listing issue links.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_issue_links(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let issue_iid = create_test_issue(&ctx, &project_path, "Issue for links").await;

    let result = ctx
        .client
        .call_tool_json(
            "list_issue_links",
            json!({
                "project": project_path,
                "issue_iid": issue_iid
            }),
        )
        .await
        .expect("Failed to list issue links");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating an issue link.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_issue_link(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create two issues to link
    let issue1_iid = create_test_issue(&ctx, &project_path, "Source issue").await;
    let issue2_iid = create_test_issue(&ctx, &project_path, "Target issue").await;

    // Create a link between them
    let result = ctx
        .client
        .call_tool_json(
            "create_issue_link",
            json!({
                "project": project_path,
                "issue_iid": issue1_iid,
                "target_project": project_path,
                "target_issue_iid": issue2_iid
            }),
        )
        .await
        .expect("Failed to create issue link");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting an issue link.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_issue_link(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create two issues to link
    let issue1_iid = create_test_issue(&ctx, &project_path, "Source issue").await;
    let issue2_iid = create_test_issue(&ctx, &project_path, "Target issue").await;

    // Create a link
    let _ = ctx
        .client
        .call_tool_json(
            "create_issue_link",
            json!({
                "project": project_path,
                "issue_iid": issue1_iid,
                "target_project": project_path,
                "target_issue_iid": issue2_iid
            }),
        )
        .await
        .expect("Failed to create issue link");

    // Get the link ID from list_issue_links (create response doesn't include it)
    let links = ctx
        .client
        .call_tool_json(
            "list_issue_links",
            json!({
                "project": project_path,
                "issue_iid": issue1_iid
            }),
        )
        .await
        .expect("Failed to list issue links");

    let link_id = links
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|link| link.get("issue_link_id"))
        .and_then(|v| v.as_u64())
        .expect("No link ID found in issue links");

    // Delete the link
    let result = ctx
        .client
        .call_tool(
            "delete_issue_link",
            json!({
                "project": project_path,
                "issue_iid": issue1_iid,
                "issue_link_id": link_id
            }),
        )
        .await
        .expect("Failed to delete issue link");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
