//! Commit tools
//!
//! Tools for viewing and managing repository commits.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List commits in a repository
#[gitlab_tool(
    name = "list_commits",
    description = "List commits in a repository with optional filtering",
    category = "commits",
    operation = "read"
)]
pub struct ListCommits {
    /// Project path or ID
    pub project: String,
    /// Branch, tag, or commit SHA to list commits from
    #[serde(default)]
    pub ref_name: Option<String>,
    /// File path to filter commits by
    #[serde(default)]
    pub path: Option<String>,
    /// Only commits after this date (ISO 8601 format)
    #[serde(default)]
    pub since: Option<String>,
    /// Only commits before this date (ISO 8601 format)
    #[serde(default)]
    pub until: Option<String>,
    /// Include commit stats
    #[serde(default)]
    pub with_stats: bool,
    /// Number of commits per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListCommits {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional_encoded("ref_name", self.ref_name.as_ref())
            .optional_encoded("path", self.path.as_ref())
            .optional_encoded("since", self.since.as_ref())
            .optional_encoded("until", self.until.as_ref())
            .optional("with_stats", self.with_stats.then_some("true"))
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/repository/commits{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific commit
#[gitlab_tool(
    name = "get_commit",
    description = "Get details of a specific commit",
    category = "commits",
    operation = "read"
)]
pub struct GetCommit {
    /// Project path or ID
    pub project: String,
    /// Commit SHA
    pub sha: String,
    /// Include commit stats
    #[serde(default)]
    pub stats: bool,
}

#[async_trait]
impl ToolExecutor for GetCommit {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("stats", self.stats.then_some("true"))
            .build();
        let endpoint = format!(
            "/projects/{}/repository/commits/{}{}",
            project, self.sha, query
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get commit diff
#[gitlab_tool(
    name = "get_commit_diff",
    description = "Get the diff of a specific commit",
    category = "commits",
    operation = "read"
)]
pub struct GetCommitDiff {
    /// Project path or ID
    pub project: String,
    /// Commit SHA
    pub sha: String,
}

#[async_trait]
impl ToolExecutor for GetCommitDiff {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/repository/commits/{}/diff", project, self.sha);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get commit comments
#[gitlab_tool(
    name = "get_commit_comments",
    description = "Get comments on a specific commit",
    category = "commits",
    operation = "read"
)]
pub struct GetCommitComments {
    /// Project path or ID
    pub project: String,
    /// Commit SHA
    pub sha: String,
}

#[async_trait]
impl ToolExecutor for GetCommitComments {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/repository/commits/{}/comments",
            project, self.sha
        );

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Add a comment to a commit
#[gitlab_tool(
    name = "create_commit_comment",
    description = "Add a comment to a specific commit",
    category = "commits",
    operation = "write"
)]
pub struct CreateCommitComment {
    /// Project path or ID
    pub project: String,
    /// Commit SHA
    pub sha: String,
    /// Comment text
    pub note: String,
    /// File path to comment on (optional, for line comments)
    #[serde(default)]
    pub path: Option<String>,
    /// Line number to comment on (required if path is set)
    #[serde(default)]
    pub line: Option<u32>,
    /// Line type: "new" or "old"
    #[serde(default)]
    pub line_type: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateCommitComment {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/repository/commits/{}/comments",
            project, self.sha
        );

        let mut body = serde_json::json!({
            "note": self.note,
        });

        if let Some(ref path) = self.path {
            body["path"] = serde_json::Value::String(path.clone());
        }
        if let Some(line) = self.line {
            body["line"] = serde_json::Value::Number(line.into());
        }
        if let Some(ref line_type) = self.line_type {
            body["line_type"] = serde_json::Value::String(line_type.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Cherry-pick a commit to a branch
#[gitlab_tool(
    name = "cherry_pick_commit",
    description = "Cherry-pick a commit to a target branch",
    category = "commits",
    operation = "write"
)]
pub struct CherryPickCommit {
    /// Project path or ID
    pub project: String,
    /// Commit SHA to cherry-pick
    pub sha: String,
    /// Target branch name
    pub branch: String,
    /// Automatically resolve conflicts by accepting source version
    #[serde(default)]
    pub dry_run: bool,
}

#[async_trait]
impl ToolExecutor for CherryPickCommit {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/repository/commits/{}/cherry_pick",
            project, self.sha
        );

        let mut body = serde_json::json!({
            "branch": self.branch,
        });

        if self.dry_run {
            body["dry_run"] = serde_json::Value::Bool(true);
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Revert a commit
#[gitlab_tool(
    name = "revert_commit",
    description = "Revert a commit to a target branch",
    category = "commits",
    operation = "write"
)]
pub struct RevertCommit {
    /// Project path or ID
    pub project: String,
    /// Commit SHA to revert
    pub sha: String,
    /// Target branch name
    pub branch: String,
    /// Perform a dry run without actually reverting
    #[serde(default)]
    pub dry_run: bool,
}

#[async_trait]
impl ToolExecutor for RevertCommit {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/repository/commits/{}/revert",
            project, self.sha
        );

        let mut body = serde_json::json!({
            "branch": self.branch,
        });

        if self.dry_run {
            body["dry_run"] = serde_json::Value::Bool(true);
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}
