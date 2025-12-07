//! E2E tests for commit tools.
//!
//! Tests: list_commits, get_commit, get_commit_diff, compare_refs,
//!        cherry_pick_commit, revert_commit, create_commit_comment, get_commit_comments

mod common;

use rstest::rstest;
use serde_json::json;
use tanuki_mcp_e2e::{TestContextBuilder, TransportKind};

/// Test listing commits.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_list_commits(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    let result = ctx
        .client
        .call_tool_json("list_commits", json!({ "project": project_path }))
        .await
        .expect("Failed to list commits");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let commits = result.as_array().unwrap();
    // Project should have at least one commit (initial commit)
    assert!(!commits.is_empty(), "Expected at least one commit");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting a specific commit.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_commit(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // First get the list of commits to get a valid SHA
    let commits = ctx
        .client
        .call_tool_json("list_commits", json!({ "project": project_path }))
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    let result = ctx
        .client
        .call_tool_json(
            "get_commit",
            json!({
                "project": project_path,
                "sha": commit_sha
            }),
        )
        .await
        .expect("Failed to get commit");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some(commit_sha));

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting commit diff.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_commit_diff(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Get a commit SHA
    let commits = ctx
        .client
        .call_tool_json("list_commits", json!({ "project": project_path }))
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    let result = ctx
        .client
        .call_tool_json(
            "get_commit_diff",
            json!({
                "project": project_path,
                "sha": commit_sha
            }),
        )
        .await
        .expect("Failed to get commit diff");

    assert!(result.is_array(), "Expected array, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test comparing refs.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_compare_refs(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create a new branch with a file change
    let branch_name = common::unique_name("compare-branch");

    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": branch_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create branch");

    // Add a file to the new branch
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "compare-test.txt",
                "branch": branch_name,
                "content": "Test content for compare",
                "commit_message": "Add compare test file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Compare the branches
    let result = ctx
        .client
        .call_tool_json(
            "compare_refs",
            json!({
                "project": project_path,
                "from": "main",
                "to": branch_name
            }),
        )
        .await
        .expect("Failed to compare refs");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("commits").is_some(), "Expected commits field");
    assert!(result.get("diffs").is_some(), "Expected diffs field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test cherry-picking a commit.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_cherry_pick_commit(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create a source branch with a commit
    let source_branch = common::unique_name("cherry-source");
    let target_branch = common::unique_name("cherry-target");

    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": source_branch,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create source branch");

    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": target_branch,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create target branch");

    // Add a file to source branch
    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "cherry-pick-test.txt",
                "branch": source_branch,
                "content": "Cherry pick content",
                "commit_message": "Add cherry pick test file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Get the commit SHA from the source branch
    let commits = ctx
        .client
        .call_tool_json(
            "list_commits",
            json!({
                "project": project_path,
                "ref_name": source_branch
            }),
        )
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    // Cherry-pick to target branch
    let result = ctx
        .client
        .call_tool_json(
            "cherry_pick_commit",
            json!({
                "project": project_path,
                "sha": commit_sha,
                "branch": target_branch
            }),
        )
        .await
        .expect("Failed to cherry-pick commit");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test reverting a commit.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_revert_commit(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Create a branch and add a file
    let branch_name = common::unique_name("revert-branch");

    let _ = ctx
        .client
        .call_tool_json(
            "create_branch",
            json!({
                "project": project_path,
                "branch": branch_name,
                "ref_name": "main"
            }),
        )
        .await
        .expect("Failed to create branch");

    let _ = ctx
        .client
        .call_tool_json(
            "create_or_update_file",
            json!({
                "project": project_path,
                "file_path": "revert-test.txt",
                "branch": branch_name,
                "content": "Content to revert",
                "commit_message": "Add revert test file"
            }),
        )
        .await
        .expect("Failed to create file");

    // Get the commit SHA
    let commits = ctx
        .client
        .call_tool_json(
            "list_commits",
            json!({
                "project": project_path,
                "ref_name": branch_name
            }),
        )
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    // Revert the commit
    let result = ctx
        .client
        .call_tool_json(
            "revert_commit",
            json!({
                "project": project_path,
                "sha": commit_sha,
                "branch": branch_name
            }),
        )
        .await
        .expect("Failed to revert commit");

    assert!(result.is_object(), "Expected object, got: {:?}", result);

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test creating a commit comment.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_create_commit_comment(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Get a commit SHA
    let commits = ctx
        .client
        .call_tool_json("list_commits", json!({ "project": project_path }))
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    let result = ctx
        .client
        .call_tool_json(
            "create_commit_comment",
            json!({
                "project": project_path,
                "sha": commit_sha,
                "note": "Test commit comment"
            }),
        )
        .await
        .expect("Failed to create commit comment");

    assert!(result.is_object(), "Expected object, got: {:?}", result);
    assert!(result.get("note").is_some(), "Expected note field");

    ctx.cleanup().await.expect("Cleanup failed");
}

/// Test getting commit comments.
#[rstest]
#[case::stdio(TransportKind::Stdio)]
#[case::http(TransportKind::Http)]
#[tokio::test]
async fn test_get_commit_comments(#[case] transport: TransportKind) {
    common::init_tracing();

    let Some(ctx) = TestContextBuilder::new(transport)
        .with_project()
        .build()
        .await
        .expect("Failed to create context") else { return; };

    let project_path = ctx.project_path.clone().expect("No project path");

    // Get a commit SHA
    let commits = ctx
        .client
        .call_tool_json("list_commits", json!({ "project": project_path }))
        .await
        .expect("Failed to list commits");

    let commit_sha = commits
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .expect("No commit found");

    // Create a comment first
    let _ = ctx
        .client
        .call_tool_json(
            "create_commit_comment",
            json!({
                "project": project_path,
                "sha": commit_sha,
                "note": "Test comment for listing"
            }),
        )
        .await
        .expect("Failed to create commit comment");

    // Get comments
    let result = ctx
        .client
        .call_tool_json(
            "get_commit_comments",
            json!({
                "project": project_path,
                "sha": commit_sha
            }),
        )
        .await
        .expect("Failed to get commit comments");

    assert!(result.is_array(), "Expected array, got: {:?}", result);
    let comments = result.as_array().unwrap();
    assert!(!comments.is_empty(), "Expected at least one comment");

    ctx.cleanup().await.expect("Cleanup failed");
}
