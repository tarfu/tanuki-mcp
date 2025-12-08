//! Search tools
//!
//! Tools for searching across GitLab (global, project, and group search).

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
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
        let mut params = vec![
            format!("scope={}", urlencoding::encode(&self.scope)),
            format!("search={}", urlencoding::encode(&self.search)),
        ];

        if let Some(ref state) = self.state {
            params.push(format!("state={}", urlencoding::encode(state)));
        }
        if let Some(confidential) = self.confidential {
            params.push(format!("confidential={}", confidential));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let endpoint = format!("/search?{}", params.join("&"));
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
        let mut params = vec![
            format!("scope={}", urlencoding::encode(&self.scope)),
            format!("search={}", urlencoding::encode(&self.search)),
        ];

        if let Some(ref ref_name) = self.ref_name {
            params.push(format!("ref={}", urlencoding::encode(ref_name)));
        }
        if let Some(ref state) = self.state {
            params.push(format!("state={}", urlencoding::encode(state)));
        }
        if let Some(confidential) = self.confidential {
            params.push(format!("confidential={}", confidential));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let endpoint = format!("/projects/{}/search?{}", project, params.join("&"));
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
        let mut params = vec![
            format!("scope={}", urlencoding::encode(&self.scope)),
            format!("search={}", urlencoding::encode(&self.search)),
        ];

        if let Some(ref state) = self.state {
            params.push(format!("state={}", urlencoding::encode(state)));
        }
        if let Some(confidential) = self.confidential {
            params.push(format!("confidential={}", confidential));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let endpoint = format!("/groups/{}/search?{}", group, params.join("&"));
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
