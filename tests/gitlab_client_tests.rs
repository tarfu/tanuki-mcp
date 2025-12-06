//! GitLab client integration tests with mock server

use serde_json::json;
use tanuki_mcp::auth::PatProvider;
use tanuki_mcp::config::GitLabConfig;
use tanuki_mcp::error::GitLabError;
use tanuki_mcp::gitlab::GitLabClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a test client pointing to mock server
fn create_test_client(mock_server: &MockServer, token: &str) -> GitLabClient {
    let config = GitLabConfig {
        url: mock_server.uri(),
        token: Some(token.to_string()),
        api_version: "v4".to_string(),
        timeout_secs: 30,
        max_retries: 0, // No retries for tests
        verify_ssl: true,
    };
    let auth = PatProvider::new(token.to_string()).unwrap();
    GitLabClient::new(&config, Box::new(auth)).unwrap()
}

#[tokio::test]
async fn test_get_request_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "test-project",
            "path_with_namespace": "group/test-project"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: serde_json::Value = client.get("/projects/123").await.unwrap();

    assert_eq!(result["id"], 123);
    assert_eq!(result["name"], "test-project");
}

#[tokio::test]
async fn test_post_request_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/123/issues"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 456,
            "iid": 1,
            "title": "New Issue",
            "state": "opened"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let body = json!({"title": "New Issue", "description": "Test description"});
    let result: serde_json::Value = client.post("/projects/123/issues", &body).await.unwrap();

    assert_eq!(result["id"], 456);
    assert_eq!(result["title"], "New Issue");
}

#[tokio::test]
async fn test_put_request_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/123/issues/1"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 456,
            "iid": 1,
            "title": "Updated Issue",
            "state": "opened"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let body = json!({"title": "Updated Issue"});
    let result: serde_json::Value = client.put("/projects/123/issues/1", &body).await.unwrap();

    assert_eq!(result["title"], "Updated Issue");
}

#[tokio::test]
async fn test_delete_request_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/123/issues/1"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result = client.delete("/projects/123/issues/1").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unauthorized_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "message": "401 Unauthorized"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "invalid-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/123").await;

    assert!(matches!(result, Err(GitLabError::Unauthorized)));
}

#[tokio::test]
async fn test_forbidden_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "message": "403 Forbidden"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/123").await;

    assert!(matches!(result, Err(GitLabError::Forbidden { .. })));
}

#[tokio::test]
async fn test_not_found_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "404 Project Not Found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/999").await;

    assert!(matches!(result, Err(GitLabError::NotFound { .. })));
}

#[tokio::test]
async fn test_rate_limited_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"message": "Rate limit exceeded"}))
                .insert_header("Retry-After", "60"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/123").await;

    assert!(matches!(result, Err(GitLabError::RateLimited { .. })));
}

#[tokio::test]
async fn test_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/123").await;

    assert!(matches!(result, Err(GitLabError::Api { status: 500, .. })));
}

#[tokio::test]
async fn test_invalid_json_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/123"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: Result<serde_json::Value, _> = client.get("/projects/123").await;

    assert!(matches!(result, Err(GitLabError::InvalidResponse(_))));
}

#[tokio::test]
async fn test_project_path_encoding() {
    let mock_server = MockServer::start().await;

    // The client should URL-encode the project path
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Fsubgroup%2Fproject"))
        .and(header("PRIVATE-TOKEN", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "path_with_namespace": "group/subgroup/project"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let encoded = GitLabClient::encode_project("group/subgroup/project");
    let result: serde_json::Value = client.get(&format!("/projects/{}", encoded)).await.unwrap();

    assert_eq!(result["path_with_namespace"], "group/subgroup/project");
}

#[tokio::test]
async fn test_pagination_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(wiremock::matchers::query_param("page", "2"))
        .and(wiremock::matchers::query_param("per_page", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 21, "name": "project-21"},
            {"id": 22, "name": "project-22"}
        ])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result: serde_json::Value = client.get("/projects?page=2&per_page=20").await.unwrap();

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_empty_response_body_on_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/123/issues/1"))
        .respond_with(ResponseTemplate::new(204)) // No body
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let result = client.delete("/projects/123/issues/1").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_error_with_detailed_message() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/123/issues"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "message": {
                "title": ["can't be blank"],
                "description": ["is too long (maximum is 1000000 characters)"]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server, "test-token");
    let body = json!({});
    let result: Result<serde_json::Value, _> = client.post("/projects/123/issues", &body).await;

    match result {
        Err(GitLabError::Api {
            status: 400,
            message,
        }) => {
            assert!(message.contains("title"));
        }
        _ => panic!("Expected Api error with status 400"),
    }
}
