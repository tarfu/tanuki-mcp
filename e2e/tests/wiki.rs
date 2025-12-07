//! E2E tests for wiki tools.
//!
//! Tests: list_wiki_pages, get_wiki_page, create_wiki_page, update_wiki_page, delete_wiki_page

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing wiki pages.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_wiki_pages(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let page_title = common::unique_name("list-wiki");

    // Create a wiki page first
    let _ = ctx
        .client
        .call_tool_json(
            "create_wiki_page",
            json!({
                "project": project_path,
                "title": page_title,
                "content": "Wiki page content"
            }),
        )
        .await
        .expect("Failed to create wiki page");

    let result = ctx
        .client
        .call_tool_json("list_wiki_pages", json!({ "project": project_path }))
        .await
        .expect("Failed to list wiki pages");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let pages = result.as_array().unwrap();
    assert!(!pages.is_empty(), "Expected at least one wiki page");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific wiki page.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_wiki_page(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let page_title = common::unique_name("get-wiki");

    let created = ctx
        .client
        .call_tool_json(
            "create_wiki_page",
            json!({
                "project": project_path,
                "title": page_title,
                "content": "Content to get"
            }),
        )
        .await
        .expect("Failed to create wiki page");

    let slug = created
        .get("slug")
        .and_then(|v| v.as_str())
        .expect("No slug");

    let result = ctx
        .client
        .call_tool_json(
            "get_wiki_page",
            json!({
                "project": project_path,
                "slug": slug
            }),
        )
        .await
        .expect("Failed to get wiki page");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("slug").and_then(|v| v.as_str()), Some(slug));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a wiki page.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_wiki_page(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let page_title = common::unique_name("create-wiki");

    let result = ctx
        .client
        .call_tool_json(
            "create_wiki_page",
            json!({
                "project": project_path,
                "title": page_title,
                "content": "# New Wiki Page\n\nThis is the content."
            }),
        )
        .await
        .expect("Failed to create wiki page");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some(page_title.as_str())
    );
    assert!(result.get("slug").is_some(), "Expected slug field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a wiki page.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_wiki_page(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let page_title = common::unique_name("update-wiki");

    let created = ctx
        .client
        .call_tool_json(
            "create_wiki_page",
            json!({
                "project": project_path,
                "title": page_title,
                "content": "Original content"
            }),
        )
        .await
        .expect("Failed to create wiki page");

    let slug = created
        .get("slug")
        .and_then(|v| v.as_str())
        .expect("No slug");

    let result = ctx
        .client
        .call_tool_json(
            "update_wiki_page",
            json!({
                "project": project_path,
                "slug": slug,
                "title": "Updated Title",
                "content": "Updated content"
            }),
        )
        .await
        .expect("Failed to update wiki page");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(
        result.get("title").and_then(|v| v.as_str()),
        Some("Updated Title")
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a wiki page.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_wiki_page(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let page_title = common::unique_name("delete-wiki");

    let created = ctx
        .client
        .call_tool_json(
            "create_wiki_page",
            json!({
                "project": project_path,
                "title": page_title,
                "content": "Page to delete"
            }),
        )
        .await
        .expect("Failed to create wiki page");

    let slug = created
        .get("slug")
        .and_then(|v| v.as_str())
        .expect("No slug");

    let result = ctx
        .client
        .call_tool(
            "delete_wiki_page",
            json!({
                "project": project_path,
                "slug": slug
            }),
        )
        .await
        .expect("Failed to delete wiki page");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}
