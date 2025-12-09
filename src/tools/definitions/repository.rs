//! Repository tools
//!
//! Tools for interacting with GitLab repository files, trees, and content.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use base64::Engine;
use tanuki_mcp_macros::gitlab_tool;

/// Get repository file contents
#[gitlab_tool(
    name = "get_repository_file",
    description = "Get the contents of a file from a repository at a specific ref",
    category = "repository",
    operation = "read"
)]
pub struct GetRepositoryFile {
    /// Project path or ID (e.g., "group/project" or "123")
    pub project: String,
    /// Path to the file in the repository
    pub file_path: String,
    /// Branch, tag, or commit SHA (default: default branch)
    #[serde(default)]
    pub ref_name: Option<String>,
}

#[async_trait]
impl ToolExecutor for GetRepositoryFile {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let file_path = urlencoding::encode(&self.file_path);

        let mut endpoint = format!("/projects/{}/repository/files/{}", project, file_path);

        if let Some(ref ref_name) = self.ref_name {
            endpoint.push_str(&format!("?ref={}", urlencoding::encode(ref_name)));
        } else {
            endpoint.push_str("?ref=HEAD");
        }

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        // Decode base64 content if present
        if let Some(content) = result.get("content").and_then(|c| c.as_str())
            && let Some(encoding) = result.get("encoding").and_then(|e| e.as_str())
            && encoding == "base64"
            && let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(content)
            && let Ok(text) = String::from_utf8(decoded)
        {
            let mut output = result.clone();
            output["content"] = serde_json::Value::String(text);
            output["encoding"] = serde_json::Value::String("text".to_string());
            return ToolOutput::json_value(output);
        }

        ToolOutput::json_value(result)
    }
}

/// Get repository tree (file listing)
#[gitlab_tool(
    name = "get_repository_tree",
    description = "Get the repository tree (list of files and directories) for a path",
    category = "repository",
    operation = "read"
)]
pub struct GetRepositoryTree {
    /// Project path or ID
    pub project: String,
    /// Path inside the repository (empty string for root)
    #[serde(default)]
    pub path: Option<String>,
    /// Branch, tag, or commit SHA (default: default branch)
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Include subdirectories recursively
    #[serde(default)]
    pub recursive: bool,
    /// Number of items per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for GetRepositoryTree {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional_encoded("path", self.path.as_ref())
            .optional_encoded("ref", self.ref_name.as_ref())
            .optional("recursive", self.recursive.then_some("true"))
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/repository/tree{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Create or update a file in the repository
#[gitlab_tool(
    name = "create_or_update_file",
    description = "Create a new file or update an existing file in the repository",
    category = "repository",
    operation = "write"
)]
pub struct CreateOrUpdateFile {
    /// Project path or ID
    pub project: String,
    /// Path for the file in the repository
    pub file_path: String,
    /// Branch name to commit to
    pub branch: String,
    /// File content
    pub content: String,
    /// Commit message
    pub commit_message: String,
    /// Start a new branch from this ref (optional)
    #[serde(default)]
    pub start_branch: Option<String>,
    /// Author email (optional)
    #[serde(default)]
    pub author_email: Option<String>,
    /// Author name (optional)
    #[serde(default)]
    pub author_name: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateOrUpdateFile {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let file_path = urlencoding::encode(&self.file_path);

        // Check if file exists to determine if we should create or update
        let check_endpoint = format!(
            "/projects/{}/repository/files/{}?ref={}",
            project,
            file_path,
            urlencoding::encode(&self.branch)
        );

        let file_exists = ctx.gitlab.get_json(&check_endpoint).await.is_ok();

        let endpoint = format!("/projects/{}/repository/files/{}", project, file_path);

        let mut body = serde_json::json!({
            "branch": self.branch,
            "content": self.content,
            "commit_message": self.commit_message,
        });

        if let Some(ref start_branch) = self.start_branch {
            body["start_branch"] = serde_json::Value::String(start_branch.clone());
        }
        if let Some(ref email) = self.author_email {
            body["author_email"] = serde_json::Value::String(email.clone());
        }
        if let Some(ref name) = self.author_name {
            body["author_name"] = serde_json::Value::String(name.clone());
        }

        let result: serde_json::Value = if file_exists {
            ctx.gitlab.put(&endpoint, &body).await?
        } else {
            ctx.gitlab.post(&endpoint, &body).await?
        };

        ToolOutput::json_value(result)
    }
}

/// Delete a file from the repository
#[gitlab_tool(
    name = "delete_repository_file",
    description = "Delete a file from the repository",
    category = "repository",
    operation = "delete"
)]
pub struct DeleteRepositoryFile {
    /// Project path or ID
    pub project: String,
    /// Path to the file to delete
    pub file_path: String,
    /// Branch name to commit to
    pub branch: String,
    /// Commit message
    pub commit_message: String,
    /// Start a new branch from this ref (optional)
    #[serde(default)]
    pub start_branch: Option<String>,
}

#[async_trait]
impl ToolExecutor for DeleteRepositoryFile {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let file_path = urlencoding::encode(&self.file_path);
        let endpoint = format!("/projects/{}/repository/files/{}", project, file_path);

        let mut body = serde_json::json!({
            "branch": self.branch,
            "commit_message": self.commit_message,
        });

        if let Some(ref start_branch) = self.start_branch {
            body["start_branch"] = serde_json::Value::String(start_branch.clone());
        }

        ctx.gitlab.delete_with_body(&endpoint, &body).await?;

        Ok(ToolOutput::text("File deleted successfully"))
    }
}

/// Search repository for files matching a pattern
#[gitlab_tool(
    name = "search_repository",
    description = "Search for files in the repository by filename or content",
    category = "repository",
    operation = "read"
)]
pub struct SearchRepository {
    /// Project path or ID
    pub project: String,
    /// Search query
    pub search: String,
    /// Branch, tag, or commit SHA (default: default branch)
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Number of results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for SearchRepository {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("scope", "blobs")
            .param("search", urlencoding::encode(&self.search))
            .optional_encoded("ref", self.ref_name.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/search{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get file blame information
#[gitlab_tool(
    name = "get_file_blame",
    description = "Get blame information for a file showing line-by-line commit history",
    category = "repository",
    operation = "read"
)]
pub struct GetFileBlame {
    /// Project path or ID
    pub project: String,
    /// Path to the file
    pub file_path: String,
    /// Branch, tag, or commit SHA (default: default branch)
    #[serde(default)]
    pub ref_name: Option<String>,
}

#[async_trait]
impl ToolExecutor for GetFileBlame {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let file_path = urlencoding::encode(&self.file_path);

        let mut endpoint = format!("/projects/{}/repository/files/{}/blame", project, file_path);

        if let Some(ref ref_name) = self.ref_name {
            endpoint.push_str(&format!("?ref={}", urlencoding::encode(ref_name)));
        } else {
            endpoint.push_str("?ref=HEAD");
        }

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Compare branches, tags, or commits
#[gitlab_tool(
    name = "compare_refs",
    description = "Compare two branches, tags, or commits in a repository",
    category = "repository",
    operation = "read"
)]
pub struct CompareRefs {
    /// Project path or ID
    pub project: String,
    /// Source branch/tag/commit
    pub from: String,
    /// Target branch/tag/commit
    pub to: String,
    /// Include full file diffs
    #[serde(default)]
    pub straight: bool,
}

#[async_trait]
impl ToolExecutor for CompareRefs {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("from", urlencoding::encode(&self.from))
            .param("to", urlencoding::encode(&self.to))
            .optional("straight", self.straight.then_some("true"))
            .build();

        let endpoint = format!("/projects/{}/repository/compare{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all repository tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<GetRepositoryFile>();
    registry.register::<GetRepositoryTree>();
    registry.register::<CreateOrUpdateFile>();
    registry.register::<DeleteRepositoryFile>();
    registry.register::<SearchRepository>();
    registry.register::<GetFileBlame>();
    registry.register::<CompareRefs>();
}
