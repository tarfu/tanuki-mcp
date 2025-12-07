//! E2E tests for job tools.
//!
//! Tests: list_pipeline_jobs, get_job, get_job_log, retry_job, cancel_job, play_job
//!
//! Note: Job tests require a project with CI/CD configuration.
//! Some tests may be limited without actual runners.

use crate::common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Helper to create a project with .gitlab-ci.yml for job tests.
async fn setup_project_with_ci(ctx: &tanuki_mcp_e2e::TestContext, project_path: &str) {
    let ci_content = r#"
stages:
  - test

test_job:
  stage: test
  script:
    - echo "Hello from CI"
  when: manual

auto_job:
  stage: test
  script:
    - echo "Auto job"
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

/// Helper to create a pipeline and get job ID.
async fn create_pipeline_and_get_job(
    ctx: &tanuki_mcp_e2e::TestContext,
    project_path: &str,
) -> (i64, i64) {
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

    // Wait for jobs to be created
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let jobs = ctx
        .client
        .call_tool_json(
            "list_pipeline_jobs",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to list jobs");

    let job_id = jobs
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64())
        .expect("No job found");

    (pipeline_id, job_id)
}

/// Helper to wait for a job to complete (success, failed, or canceled).
async fn wait_for_job_completion(
    ctx: &tanuki_mcp_e2e::TestContext,
    project_path: &str,
    job_id: i64,
    timeout_secs: u64,
) -> serde_json::Value {
    let start = std::time::Instant::now();
    loop {
        let job = ctx
            .client
            .call_tool_json(
                "get_job",
                json!({
                    "project": project_path,
                    "job_id": job_id
                }),
            )
            .await
            .expect("Failed to get job");

        let status = job
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        // Check if job has finished (not pending, running, or created)
        match status {
            "success" | "failed" | "canceled" | "skipped" => return job,
            _ => {
                if start.elapsed().as_secs() > timeout_secs {
                    panic!(
                        "Timeout waiting for job {} to complete. Current status: {}",
                        job_id, status
                    );
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// Test listing pipeline jobs.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_pipeline_jobs(#[case] transport: TransportKind) {
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

    // Wait for jobs to be created
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = ctx
        .client
        .call_tool_json(
            "list_pipeline_jobs",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to list jobs");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let jobs = result.as_array().unwrap();
    assert!(!jobs.is_empty(), "Expected at least one job");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific job.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_job(#[case] transport: TransportKind) {
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
    let (_, job_id) = create_pipeline_and_get_job(&ctx, &project_path).await;

    let result = ctx
        .client
        .call_tool_json(
            "get_job",
            json!({
                "project": project_path,
                "job_id": job_id
            }),
        )
        .await
        .expect("Failed to get job");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_i64()), Some(job_id));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting job log.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_job_log(#[case] transport: TransportKind) {
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
    let (_, job_id) = create_pipeline_and_get_job(&ctx, &project_path).await;

    // Wait for the job to complete so we have logs
    wait_for_job_completion(&ctx, &project_path, job_id, 60).await;

    // Get job log - returns text content, not JSON
    let result = ctx
        .client
        .call_tool(
            "get_job_log",
            json!({
                "project": project_path,
                "job_id": job_id
            }),
        )
        .await
        .expect("Failed to get job log");

    // Job log returns text content, not JSON
    assert!(
        result.is_error != Some(true),
        "Get job log returned error: {:?}",
        result
    );

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test retrying a job.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_retry_job(#[case] transport: TransportKind) {
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
    let (_, job_id) = create_pipeline_and_get_job(&ctx, &project_path).await;

    // Wait for job to complete - can only retry finished jobs
    wait_for_job_completion(&ctx, &project_path, job_id, 60).await;

    let result = ctx
        .client
        .call_tool_json(
            "retry_job",
            json!({
                "project": project_path,
                "job_id": job_id
            }),
        )
        .await
        .expect("Failed to retry job");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test cancelling a job.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_cancel_job(#[case] transport: TransportKind) {
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
    let (_, job_id) = create_pipeline_and_get_job(&ctx, &project_path).await;

    let result = ctx
        .client
        .call_tool_json(
            "cancel_job",
            json!({
                "project": project_path,
                "job_id": job_id
            }),
        )
        .await
        .expect("Failed to cancel job");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test playing a manual job.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_play_job(#[case] transport: TransportKind) {
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

    // Wait for jobs to be created
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Find the manual job
    let jobs = ctx
        .client
        .call_tool_json(
            "list_pipeline_jobs",
            json!({
                "project": project_path,
                "pipeline_id": pipeline_id
            }),
        )
        .await
        .expect("Failed to list jobs");

    let manual_job = jobs
        .as_array()
        .and_then(|arr| {
            arr.iter().find(|j| {
                j.get("name").and_then(|n| n.as_str()) == Some("test_job")
                    && j.get("status").and_then(|s| s.as_str()) == Some("manual")
            })
        })
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64());

    if let Some(job_id) = manual_job {
        let result = ctx
            .client
            .call_tool_json(
                "play_job",
                json!({
                    "project": project_path,
                    "job_id": job_id
                }),
            )
            .await
            .expect("Failed to play job");

        assert!(result.is_object(), "Expected object, got: {:?}", result);
    }

    ctx.cleanup().await.expect("Cleanup failed");
}
