//! E2E tests for pipeline tools.
//!
//! Tests: list_pipelines, get_pipeline, create_pipeline, retry_pipeline,
//!        cancel_pipeline, delete_pipeline, get_pipeline_variables
//!
//! Note: Pipeline tests require a project with CI/CD configuration.
//! Some tests may be limited without actual runners.

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Helper to create a project with .gitlab-ci.yml for pipeline tests.
async fn setup_project_with_ci(ctx: &tanuki_mcp_e2e::TestContext, project_path: &str) {
    let ci_content = r#"
stages:
  - test

test_job:
  stage: test
  script:
    - echo "Hello from CI"
  when: manual
"#;

    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": ".gitlab-ci.yml",
                "branch": "main",
                "content": ci_content,
                "commit_message": "Add CI configuration"
            }),
        )
        .await
        .expect("Failed to create .gitlab-ci.yml");
}

/// Test listing pipelines.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_pipelines(#[case] transport: TransportKind) {
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

    // Setup CI and trigger a pipeline
    setup_project_with_ci(&ctx, &project_path).await;

    let _ = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let result = ctx
        .client
        .call_tool_json("list_pipelines", json!({ "project": project_path }))
        .await
        .expect("Failed to list pipelines");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let pipelines = result.as_array().unwrap();
    assert!(!pipelines.is_empty(), "Expected at least one pipeline");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific pipeline.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_pipeline(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    let pipeline = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let pipeline_id = pipeline
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No pipeline id");

    let result = ctx
        .client
        .call_tool_json(
            "get_pipeline",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to get pipeline");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_i64()), Some(pipeline_id));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a pipeline.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_pipeline(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    let result = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("id").is_some(), "Expected id field");
    assert!(result.get("status").is_some(), "Expected status field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test cancelling a pipeline.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_cancel_pipeline(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    let pipeline = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let pipeline_id = pipeline
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No pipeline id");

    let result = ctx
        .client
        .call_tool_json(
            "cancel_pipeline",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to cancel pipeline");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test retrying a pipeline.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_retry_pipeline(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    let pipeline = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let pipeline_id = pipeline
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No pipeline id");

    // Wait a moment for pipeline to be in a retriable state
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "retry_pipeline",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to retry pipeline");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test deleting a pipeline.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_delete_pipeline(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    let pipeline = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let pipeline_id = pipeline
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No pipeline id");

    let result = ctx
        .client
        .call_tool(
            "delete_pipeline",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to delete pipeline");

    assert!(
        result.is_error != Some(true),
        "Delete returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting pipeline variables.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_pipeline_variables(#[case] transport: TransportKind) {
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

    setup_project_with_ci(&ctx, &project_path).await;

    // Create a pipeline with variables
    let pipeline = ctx
        .client
        .call_tool_json(
            "create_pipeline",
            json!({
                "project": project_path,
                "ref_name": "main",
                "variables": [
                    {"key": "TEST_VAR", "value": "test_value"}
                ]
            }),
        )
        .await
        .expect("Failed to create pipeline");

    let pipeline_id = pipeline
        .get("id")
        .and_then(|v| v.as_i64())
        .expect("No pipeline id");

    let result = ctx
        .client
        .call_tool_json(
            "get_pipeline_variables",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to get pipeline variables");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}
