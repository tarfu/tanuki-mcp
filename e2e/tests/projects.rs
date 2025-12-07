//! E2E tests for project tools.
//!
//! Tests: list_projects, get_project, create_project, update_project,
//!        delete_project, fork_project, list_project_members

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContext, TestContextBuilder, TransportKind};

/// Test listing projects.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_projects(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContext::new(transport)
        .await
        .expect("Failed to create context") else { return; };

    let result = ctx
        .client
        .call_tool_json("list_projects", json!({}))
        .await
        .expect("Failed to list projects");

    // Result should be an array (may be empty initially)
    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json("get_project", json!({ "project": project_path }))
        .await
        .expect("Failed to get project");

    // Verify project details
    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Missing project ID");
    assert!(result.get("name").is_some(), "Missing project name");
    assert!(result.get("path_with_namespace").is_some(), "Missing path");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContext::new(transport)
        .await
        .expect("Failed to create context") else { return; };

    let project_name = common::unique_name("test-create");

    let result = ctx
        .client
        .call_tool_json(
            "create_project",
            json!({
                "name": project_name,
                "visibility": "private",
                "description": "E2E test project"
            }),
        )
        .await
        .expect("Failed to create project");

    // Verify project was created
    assert!(result.is_object(), "Expected object, got: {:?}", result);
    let project_id = result.get("id").and_then(|v| v.as_u64());
    assert!(project_id.is_some(), "Missing project ID");

    // Cleanup - delete the created project
    if let Some(id) = project_id {
        let _ = ctx.gitlab.delete_project(&ctx.token, id).await;
    }

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test updating a project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_update_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let new_description = "Updated by E2E test";

    let result = ctx
        .client
        .call_tool_json(
            "update_project",
            json!({
                "project": project_path,
                "description": new_description
            }),
        )
        .await
        .expect("Failed to update project");

    // Verify update
    assert!(result.is_object(), "Expected object, got: {:?}", result);
    let description = result.get("description").and_then(|v| v.as_str());
    assert_eq!(description, Some(new_description));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContext::new(transport)
        .await
        .expect("Failed to create context") else { return; };

    // Create a project to delete
    let project_name = common::unique_name("test-delete");

    let create_result = ctx
        .client
        .call_tool_json(
            "create_project",
            json!({
                "name": project_name,
                "visibility": "private"
            }),
        )
        .await
        .expect("Failed to create project");

    let project_path = create_result
        .get("path_with_namespace")
        .and_then(|v| v.as_str())
        .expect("No project path");

    // Delete the project
    let result = ctx
        .client
        .call_tool("delete_project", json!({ "project": project_path }))
        .await
        .expect("Failed to delete project");

    // Delete should succeed (result may indicate accepted/scheduled)
    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test forking a project.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_fork_project(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");
    let fork_name = common::unique_name("test-fork");

    let result = ctx
        .client
        .call_tool_json(
            "fork_project",
            json!({
                "project": project_path,
                "name": fork_name
            }),
        )
        .await
        .expect("Failed to fork project");

    // Verify fork was created
    assert!(result.is_object(), "Expected object, got: {:?}", result);
    let fork_id = result.get("id").and_then(|v| v.as_u64());
    assert!(fork_id.is_some(), "Missing fork ID");

    // Cleanup - delete the fork
    if let Some(id) = fork_id {
        let _ = ctx.gitlab.delete_project(&ctx.token, id).await;
    }

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing project members.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_project_members(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json("list_project_members", json!({ "project": project_path }))
        .await
        .expect("Failed to list project members");

    // Result should be an array with at least the owner (root user)
    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let members = result.as_array().unwrap();
    assert!(!members.is_empty(), "Expected at least one member (owner)");

    // Verify member has expected fields
    let first_member = &members[0];
    assert!(first_member.get("id").is_some(), "Missing member ID");
    assert!(first_member.get("username").is_some(), "Missing username");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test listing projects with pagination.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_projects_pagination(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContext::new(transport)
        .await
        .expect("Failed to create context") else { return; };

    // List with explicit page and per_page
    let result = ctx
        .client
        .call_tool_json(
            "list_projects",
            json!({
                "page": 1,
                "per_page": 5
            }),
        )
        .await
        .expect("Failed to list projects");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
