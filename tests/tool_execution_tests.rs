//! Tool execution integration tests
//!
//! Tests individual tools with mocked GitLab API responses.

use serde_json::json;
use std::sync::Arc;
use tanuki_mcp::access_control::AccessResolver;
use tanuki_mcp::auth::PatProvider;
use tanuki_mcp::config::{AccessControlConfig, AccessLevel, GitLabConfig};
use tanuki_mcp::gitlab::GitLabClient;
use tanuki_mcp::tools::{ToolContext, ToolRegistry, definitions};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create a test GitLab client
fn create_test_gitlab(mock_server: &MockServer) -> Arc<GitLabClient> {
    let config = GitLabConfig {
        url: mock_server.uri(),
        token: Some("test-token".to_string()),
        api_version: "v4".to_string(),
        timeout_secs: 30,
        max_retries: 0,
        verify_ssl: true,
    };
    let auth = PatProvider::new("test-token".to_string()).unwrap();
    Arc::new(GitLabClient::new(&config, Box::new(auth)).unwrap())
}

/// Create a test access resolver with full access
fn create_full_access() -> Arc<AccessResolver> {
    let mut policy = AccessControlConfig::default();
    policy.all = AccessLevel::Full;
    Arc::new(AccessResolver::new(&policy).unwrap())
}

/// Create a tool context for testing
fn create_test_context(gitlab: Arc<GitLabClient>, access: Arc<AccessResolver>) -> ToolContext {
    ToolContext::new(gitlab, access, "test-request-123")
}

/// Create a registry with all tools
fn create_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    definitions::register_all_tools(&mut registry);
    registry
}

// ============================================================================
// Issue Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_issues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/issues"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "iid": 1,
                "title": "First Issue",
                "state": "opened",
                "author": {"username": "alice"}
            },
            {
                "id": 2,
                "iid": 2,
                "title": "Second Issue",
                "state": "closed",
                "author": {"username": "bob"}
            }
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry.execute("list_issues", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    let content = &result.content[0];
    match content {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("First Issue"));
            assert!(text.contains("Second Issue"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_list_issues_with_filters() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/issues"))
        .and(query_param("state", "opened"))
        .and(query_param("labels", "bug,urgent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "state": "opened",
        "labels": "bug,urgent"
    });
    let result = registry.execute("list_issues", &ctx, args).await.unwrap();
    assert!(!result.is_error);
}

#[tokio::test]
async fn test_get_issue() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/issues/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 100,
            "iid": 42,
            "title": "Specific Issue",
            "description": "Issue description",
            "state": "opened",
            "author": {"username": "alice"},
            "labels": ["bug", "priority::high"]
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "issue_iid": 42});
    let result = registry.execute("get_issue", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("Specific Issue"));
            assert!(text.contains("42"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_create_issue() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/test%2Fproject/issues"))
        .and(body_json(json!({
            "title": "New Issue",
            "description": "Issue body"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 123,
            "iid": 5,
            "title": "New Issue",
            "state": "opened"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "title": "New Issue",
        "description": "Issue body"
    });
    let result = registry.execute("create_issue", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Merge Request Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_merge_requests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "iid": 1,
                "title": "Feature MR",
                "state": "opened",
                "source_branch": "feature",
                "target_branch": "main"
            }
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("list_merge_requests", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("Feature MR"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_get_merge_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/merge_requests/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 50,
            "iid": 10,
            "title": "Big Feature",
            "description": "Adds new feature",
            "state": "merged",
            "source_branch": "feature-branch",
            "target_branch": "main"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "merge_request_iid": 10});
    let result = registry
        .execute("get_merge_request", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Repository Tools Tests
// ============================================================================

#[tokio::test]
async fn test_get_repository_file() {
    let mock_server = MockServer::start().await;

    // GitLab returns base64 encoded content
    use base64::Engine;
    let content = base64::engine::general_purpose::STANDARD.encode("Hello, World!");

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/test%2Fproject/repository/files/README.md",
        ))
        .and(query_param("ref", "main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "file_name": "README.md",
            "file_path": "README.md",
            "size": 13,
            "encoding": "base64",
            "content": content,
            "ref": "main"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    // Use ref_name, not ref (as per tool definition)
    let args = json!({
        "project": "test/project",
        "file_path": "README.md",
        "ref_name": "main"
    });
    let result = registry
        .execute("get_repository_file", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_get_repository_tree() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/repository/tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "abc123", "name": "src", "type": "tree", "path": "src"},
            {"id": "def456", "name": "README.md", "type": "blob", "path": "README.md"},
            {"id": "ghi789", "name": "Cargo.toml", "type": "blob", "path": "Cargo.toml"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("get_repository_tree", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("src"));
            assert!(text.contains("README.md"));
        }
        _ => panic!("Expected text content"),
    }
}

// ============================================================================
// Project Tools Tests
// ============================================================================

#[tokio::test]
async fn test_get_project() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "project",
            "path_with_namespace": "test/project",
            "description": "Test project",
            "default_branch": "main",
            "visibility": "private"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry.execute("get_project", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("test/project"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_list_projects() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "name": "project1", "path_with_namespace": "group/project1"},
            {"id": 2, "name": "project2", "path_with_namespace": "group/project2"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({});
    let result = registry.execute("list_projects", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Pipeline Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_pipelines() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/pipelines"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 100, "status": "success", "ref": "main", "sha": "abc123"},
            {"id": 99, "status": "failed", "ref": "feature", "sha": "def456"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("list_pipelines", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_get_pipeline() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/pipelines/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 100,
            "status": "success",
            "ref": "main",
            "sha": "abc123def456",
            "created_at": "2024-01-15T10:00:00Z",
            "finished_at": "2024-01-15T10:05:00Z"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "pipeline_id": 100});
    let result = registry.execute("get_pipeline", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Access Control Tests
// ============================================================================

#[tokio::test]
async fn test_tool_access_denied() {
    let mock_server = MockServer::start().await;

    let gitlab = create_test_gitlab(&mock_server);

    // Create read-only access
    let mut policy = AccessControlConfig::default();
    policy.all = AccessLevel::Read;
    let access = Arc::new(AccessResolver::new(&policy).unwrap());

    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    // Write operation should be denied
    let args = json!({
        "project": "test/project",
        "title": "New Issue"
    });
    let result = registry.execute("create_issue", &ctx, args).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("denied") || err.to_string().contains("Access"));
}

#[tokio::test]
async fn test_tool_not_found_error() {
    let mock_server = MockServer::start().await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({});
    let result = registry.execute("nonexistent_tool", &ctx, args).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Not Found"));
}

#[tokio::test]
async fn test_tool_invalid_arguments() {
    let mock_server = MockServer::start().await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    // Missing required 'project' field
    let args = json!({"invalid_field": "value"});
    let result = registry.execute("list_issues", &ctx, args).await;

    assert!(result.is_err());
}

// ============================================================================
// Label Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_labels() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "name": "bug", "color": "#FF0000"},
            {"id": 2, "name": "feature", "color": "#00FF00"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry.execute("list_labels", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Wiki Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_wiki_pages() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/wikis"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"slug": "home", "title": "Home", "format": "markdown"},
            {"slug": "getting-started", "title": "Getting Started", "format": "markdown"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("list_wiki_pages", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Milestone Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_milestones() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "iid": 1, "title": "v1.0", "state": "active"},
            {"id": 2, "iid": 2, "title": "v2.0", "state": "active"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("list_milestones", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

// ============================================================================
// Tag Tools Tests
// ============================================================================

#[tokio::test]
async fn test_list_tags() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/repository/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "name": "v1.0.0",
                "message": "Release v1.0.0",
                "target": "abc123",
                "commit": {"id": "abc123", "message": "Initial release"}
            },
            {
                "name": "v1.1.0",
                "message": "Release v1.1.0",
                "target": "def456",
                "commit": {"id": "def456", "message": "Minor update"}
            }
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry.execute("list_tags", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("v1.0.0"));
            assert!(text.contains("v1.1.0"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_list_tags_with_filters() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/repository/tags"))
        .and(query_param("order_by", "updated"))
        .and(query_param("sort", "desc"))
        .and(query_param("search", "v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "order_by": "updated",
        "sort": "desc",
        "search": "v1"
    });
    let result = registry.execute("list_tags", &ctx, args).await.unwrap();
    assert!(!result.is_error);
}

#[tokio::test]
async fn test_get_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/test%2Fproject/repository/tags/v1.0.0",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "v1.0.0",
            "message": "Release v1.0.0",
            "target": "abc123def456",
            "commit": {
                "id": "abc123def456",
                "message": "Initial release",
                "author_name": "Developer"
            },
            "release": {
                "tag_name": "v1.0.0",
                "description": "First stable release"
            }
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "tag_name": "v1.0.0"});
    let result = registry.execute("get_tag", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("v1.0.0"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_create_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/test%2Fproject/repository/tags"))
        .and(body_json(json!({
            "tag_name": "v2.0.0",
            "ref": "main",
            "message": "Release v2.0.0"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "name": "v2.0.0",
            "message": "Release v2.0.0",
            "target": "abc123"
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "tag_name": "v2.0.0",
        "ref_name": "main",
        "message": "Release v2.0.0"
    });
    let result = registry.execute("create_tag", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_delete_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/v4/projects/test%2Fproject/repository/tags/v1.0.0",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "tag_name": "v1.0.0"});
    let result = registry.execute("delete_tag", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("deleted"));
            assert!(text.contains("v1.0.0"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_list_protected_tags() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/protected_tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"name": "v*", "create_access_levels": [{"access_level": 40}]},
            {"name": "release-*", "create_access_levels": [{"access_level": 30}]}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project"});
    let result = registry
        .execute("list_protected_tags", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_get_protected_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/protected_tags/v%2A"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "v*",
            "create_access_levels": [{"access_level": 40, "access_level_description": "Maintainers"}]
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "name": "v*"});
    let result = registry
        .execute("get_protected_tag", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_protect_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/test%2Fproject/protected_tags"))
        .and(body_json(json!({
            "name": "release-*",
            "create_access_level": 40
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "name": "release-*",
            "create_access_levels": [{"access_level": 40}]
        })))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "name": "release-*",
        "create_access_level": 40
    });
    let result = registry.execute("protect_tag", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_unprotect_tag() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/test%2Fproject/protected_tags/v%2A"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({"project": "test/project", "name": "v*"});
    let result = registry.execute("unprotect_tag", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("unprotected"));
        }
        _ => panic!("Expected text content"),
    }
}

// ============================================================================
// Search Tools Tests
// ============================================================================

#[tokio::test]
async fn test_search_global() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "projects"))
        .and(query_param("search", "gitlab"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "name": "tanuki-mcp", "path_with_namespace": "user/tanuki-mcp"},
            {"id": 2, "name": "gitlab-runner", "path_with_namespace": "org/gitlab-runner"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "scope": "projects",
        "search": "gitlab"
    });
    let result = registry.execute("search_global", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("tanuki-mcp"));
            assert!(text.contains("gitlab-runner"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_search_global_issues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "issues"))
        .and(query_param("search", "bug"))
        .and(query_param("state", "opened"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "iid": 10, "title": "Critical bug", "state": "opened"},
            {"id": 2, "iid": 20, "title": "Minor bug fix", "state": "opened"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "scope": "issues",
        "search": "bug",
        "state": "opened"
    });
    let result = registry.execute("search_global", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_search_project() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/search"))
        .and(query_param("scope", "blobs"))
        .and(query_param("search", "TODO"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "basename": "main.rs",
                "data": "// TODO: implement this",
                "path": "src/main.rs",
                "filename": "main.rs",
                "ref": "main"
            },
            {
                "basename": "lib.rs",
                "data": "// TODO: add tests",
                "path": "src/lib.rs",
                "filename": "lib.rs",
                "ref": "main"
            }
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "scope": "blobs",
        "search": "TODO"
    });
    let result = registry
        .execute("search_project", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("main.rs"));
            assert!(text.contains("lib.rs"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_search_project_with_ref() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/test%2Fproject/search"))
        .and(query_param("scope", "commits"))
        .and(query_param("search", "fix"))
        .and(query_param("ref", "develop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "abc123", "title": "Fix bug", "author_name": "Developer"}
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "project": "test/project",
        "scope": "commits",
        "search": "fix",
        "ref_name": "develop"
    });
    let result = registry
        .execute("search_project", &ctx, args)
        .await
        .unwrap();

    assert!(!result.is_error);
}

#[tokio::test]
async fn test_search_group() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/mygroup/search"))
        .and(query_param("scope", "merge_requests"))
        .and(query_param("search", "feature"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "iid": 5,
                "title": "Add new feature",
                "state": "merged",
                "source_branch": "feature-x"
            },
            {
                "id": 2,
                "iid": 10,
                "title": "Feature improvement",
                "state": "opened",
                "source_branch": "feature-y"
            }
        ])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "group": "mygroup",
        "scope": "merge_requests",
        "search": "feature"
    });
    let result = registry.execute("search_group", &ctx, args).await.unwrap();

    assert!(!result.is_error);
    match &result.content[0] {
        tanuki_mcp::tools::ContentBlock::Text { text } => {
            assert!(text.contains("Add new feature"));
            assert!(text.contains("Feature improvement"));
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_search_group_with_state_filter() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/mygroup/search"))
        .and(query_param("scope", "issues"))
        .and(query_param("search", "critical"))
        .and(query_param("state", "opened"))
        .and(query_param("confidential", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let gitlab = create_test_gitlab(&mock_server);
    let access = create_full_access();
    let ctx = create_test_context(gitlab, access);
    let registry = create_registry();

    let args = json!({
        "group": "mygroup",
        "scope": "issues",
        "search": "critical",
        "state": "opened",
        "confidential": true
    });
    let result = registry.execute("search_group", &ctx, args).await.unwrap();

    assert!(!result.is_error);
}
