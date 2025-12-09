//! Search tools
//!
//! Tools for searching across GitLab (global, project, and group search).

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// Global search across GitLab
#[gitlab_tool(
    name = "search_global",
    description = "Search across all GitLab for projects, issues, merge requests, milestones, snippet titles, or users. Available scopes: projects, issues, merge_requests, milestones, snippet_titles, users",
    category = "search",
    operation = "read"
)]
pub struct SearchGlobal {
    /// Search scope: projects, issues, merge_requests, milestones, snippet_titles, users
    pub scope: String,
    /// Search query string
    pub search: String,
    /// Filter by state (for issues/MRs): opened, closed, all
    #[serde(default)]
    pub state: Option<String>,
    /// Filter confidential issues (true/false)
    #[serde(default)]
    pub confidential: Option<bool>,
    /// Number of results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for SearchGlobal {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = QueryBuilder::new()
            .param("scope", urlencoding::encode(&self.scope))
            .param("search", urlencoding::encode(&self.search))
            .optional_encoded("state", self.state.as_ref())
            .optional("confidential", self.confidential)
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/search{}", query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Search within a project
#[gitlab_tool(
    name = "search_project",
    description = "Search within a specific project. Available scopes: blobs (code), commits, issues, merge_requests, milestones, notes, wiki_blobs, users",
    category = "search",
    operation = "read",
    project_field = "project"
)]
pub struct SearchProject {
    /// Project path or ID
    pub project: String,
    /// Search scope: blobs, commits, issues, merge_requests, milestones, notes, wiki_blobs, users
    pub scope: String,
    /// Search query string
    pub search: String,
    /// Branch/tag ref for blob and commit searches
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Filter by state (for issues/MRs): opened, closed, all
    #[serde(default)]
    pub state: Option<String>,
    /// Filter confidential issues (true/false)
    #[serde(default)]
    pub confidential: Option<bool>,
    /// Number of results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for SearchProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("scope", urlencoding::encode(&self.scope))
            .param("search", urlencoding::encode(&self.search))
            .optional_encoded("ref", self.ref_name.as_ref())
            .optional_encoded("state", self.state.as_ref())
            .optional("confidential", self.confidential)
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/search{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Search within a group
#[gitlab_tool(
    name = "search_group",
    description = "Search within a specific group and its subgroups. Available scopes: projects, issues, merge_requests, milestones, users",
    category = "search",
    operation = "read"
)]
pub struct SearchGroup {
    /// Group path or ID
    pub group: String,
    /// Search scope: projects, issues, merge_requests, milestones, users
    pub scope: String,
    /// Search query string
    pub search: String,
    /// Filter by state (for issues/MRs): opened, closed, all
    #[serde(default)]
    pub state: Option<String>,
    /// Filter confidential issues (true/false)
    #[serde(default)]
    pub confidential: Option<bool>,
    /// Number of results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for SearchGroup {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let group = urlencoding::encode(&self.group);
        let query = QueryBuilder::new()
            .param("scope", urlencoding::encode(&self.scope))
            .param("search", urlencoding::encode(&self.search))
            .optional_encoded("state", self.state.as_ref())
            .optional("confidential", self.confidential)
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/groups/{}/search{}", group, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all search tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<SearchGlobal>();
    registry.register::<SearchProject>();
    registry.register::<SearchGroup>();
}
