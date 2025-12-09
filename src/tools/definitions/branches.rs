//! Branch tools
//!
//! Tools for managing repository branches.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List repository branches
#[gitlab_tool(
    name = "list_branches",
    description = "List branches in a repository with optional filtering",
    category = "branches",
    operation = "read"
)]
pub struct ListBranches {
    /// Project path or ID
    pub project: String,
    /// Search for branches matching this string
    #[serde(default)]
    pub search: Option<String>,
    /// Number of branches per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListBranches {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional_encoded("search", self.search.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/repository/branches{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific branch
#[gitlab_tool(
    name = "get_branch",
    description = "Get information about a specific branch",
    category = "branches",
    operation = "read"
)]
pub struct GetBranch {
    /// Project path or ID
    pub project: String,
    /// Branch name
    pub branch: String,
}

#[async_trait]
impl ToolExecutor for GetBranch {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let branch = urlencoding::encode(&self.branch);
        let endpoint = format!("/projects/{}/repository/branches/{}", project, branch);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Create a new branch
#[gitlab_tool(
    name = "create_branch",
    description = "Create a new branch from a ref (branch, tag, or commit)",
    category = "branches",
    operation = "write"
)]
pub struct CreateBranch {
    /// Project path or ID
    pub project: String,
    /// Name for the new branch
    pub branch: String,
    /// Source ref (branch name, tag, or commit SHA)
    pub ref_name: String,
}

#[async_trait]
impl ToolExecutor for CreateBranch {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/repository/branches", project);

        let body = serde_json::json!({
            "branch": self.branch,
            "ref": self.ref_name,
        });

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a branch
#[gitlab_tool(
    name = "delete_branch",
    description = "Delete a branch from the repository",
    category = "branches",
    operation = "delete"
)]
pub struct DeleteBranch {
    /// Project path or ID
    pub project: String,
    /// Branch name to delete
    pub branch: String,
}

#[async_trait]
impl ToolExecutor for DeleteBranch {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let branch = urlencoding::encode(&self.branch);
        let endpoint = format!("/projects/{}/repository/branches/{}", project, branch);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Branch '{}' deleted successfully",
            self.branch
        )))
    }
}

/// Protect a branch
#[gitlab_tool(
    name = "protect_branch",
    description = "Protect a branch with access level restrictions",
    category = "branches",
    operation = "write"
)]
pub struct ProtectBranch {
    /// Project path or ID
    pub project: String,
    /// Branch name or wildcard pattern (e.g., "main", "release-*")
    pub name: String,
    /// Push access level: 0 (no one), 30 (developers), 40 (maintainers)
    #[serde(default)]
    pub push_access_level: Option<u32>,
    /// Merge access level: 0 (no one), 30 (developers), 40 (maintainers)
    #[serde(default)]
    pub merge_access_level: Option<u32>,
    /// Allow force push
    #[serde(default)]
    pub allow_force_push: Option<bool>,
}

#[async_trait]
impl ToolExecutor for ProtectBranch {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/protected_branches", project);

        let mut body = serde_json::json!({
            "name": self.name,
        });

        if let Some(level) = self.push_access_level {
            body["push_access_level"] = serde_json::Value::Number(level.into());
        }
        if let Some(level) = self.merge_access_level {
            body["merge_access_level"] = serde_json::Value::Number(level.into());
        }
        if let Some(allow) = self.allow_force_push {
            body["allow_force_push"] = serde_json::Value::Bool(allow);
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Unprotect a branch
#[gitlab_tool(
    name = "unprotect_branch",
    description = "Remove protection from a branch",
    category = "branches",
    operation = "write"
)]
pub struct UnprotectBranch {
    /// Project path or ID
    pub project: String,
    /// Branch name or wildcard pattern
    pub name: String,
}

#[async_trait]
impl ToolExecutor for UnprotectBranch {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let name = urlencoding::encode(&self.name);
        let endpoint = format!("/projects/{}/protected_branches/{}", project, name);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Branch '{}' unprotected successfully",
            self.name
        )))
    }
}
