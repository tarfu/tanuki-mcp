//! Group tools
//!
//! Tools for managing GitLab groups.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List groups
#[gitlab_tool(
    name = "list_groups",
    description = "List groups accessible to the authenticated user",
    category = "groups",
    operation = "read"
)]
pub struct ListGroups {
    /// Search by group name or path
    #[serde(default)]
    pub search: Option<String>,
    /// Filter by groups owned by current user
    #[serde(default)]
    pub owned: bool,
    /// Filter by visibility: public, internal, private
    #[serde(default)]
    pub visibility: Option<String>,
    /// Include statistics
    #[serde(default)]
    pub statistics: bool,
    /// Sort by: name, path, id, similarity (if search provided)
    #[serde(default)]
    pub order_by: Option<String>,
    /// Sort direction: asc or desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Number of groups per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListGroups {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = QueryBuilder::new()
            .optional_encoded("search", self.search.as_ref())
            .optional("owned", self.owned.then_some("true"))
            .optional("visibility", self.visibility.as_ref())
            .optional("statistics", self.statistics.then_some("true"))
            .optional("order_by", self.order_by.as_ref())
            .optional("sort", self.sort.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/groups{}", query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific group
#[gitlab_tool(
    name = "get_group",
    description = "Get details of a specific group",
    category = "groups",
    operation = "read"
)]
pub struct GetGroup {
    /// Group ID or URL-encoded path
    pub group: String,
    /// Include statistics
    #[serde(default)]
    pub statistics: bool,
    /// Include projects in the response
    #[serde(default)]
    pub with_projects: bool,
}

#[async_trait]
impl ToolExecutor for GetGroup {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let group = GitLabClient::encode_project(&self.group);
        let query = QueryBuilder::new()
            .optional("statistics", self.statistics.then_some("true"))
            .optional("with_projects", self.with_projects.then_some("true"))
            .build();

        let endpoint = format!("/groups/{}{}", group, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// List group members
#[gitlab_tool(
    name = "list_group_members",
    description = "List members of a group",
    category = "groups",
    operation = "read"
)]
pub struct ListGroupMembers {
    /// Group ID or URL-encoded path
    pub group: String,
    /// Search by name or username
    #[serde(default)]
    pub query: Option<String>,
    /// Number of members per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListGroupMembers {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let group = GitLabClient::encode_project(&self.group);
        let query_str = QueryBuilder::new()
            .optional_encoded("query", self.query.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/groups/{}/members{}", group, query_str);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// List group projects
#[gitlab_tool(
    name = "list_group_projects",
    description = "List projects in a group",
    category = "groups",
    operation = "read"
)]
pub struct ListGroupProjects {
    /// Group ID or URL-encoded path
    pub group: String,
    /// Include projects from subgroups
    #[serde(default)]
    pub include_subgroups: bool,
    /// Filter by archived status
    #[serde(default)]
    pub archived: Option<bool>,
    /// Filter by visibility
    #[serde(default)]
    pub visibility: Option<String>,
    /// Search by project name
    #[serde(default)]
    pub search: Option<String>,
    /// Number of projects per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListGroupProjects {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let group = GitLabClient::encode_project(&self.group);
        let query = QueryBuilder::new()
            .optional(
                "include_subgroups",
                self.include_subgroups.then_some("true"),
            )
            .optional("archived", self.archived)
            .optional("visibility", self.visibility.as_ref())
            .optional_encoded("search", self.search.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/groups/{}/projects{}", group, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// List subgroups
#[gitlab_tool(
    name = "list_subgroups",
    description = "List subgroups of a group",
    category = "groups",
    operation = "read"
)]
pub struct ListSubgroups {
    /// Group ID or URL-encoded path
    pub group: String,
    /// Include statistics
    #[serde(default)]
    pub statistics: bool,
    /// Search by name
    #[serde(default)]
    pub search: Option<String>,
    /// Number of subgroups per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListSubgroups {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let group = GitLabClient::encode_project(&self.group);
        let query = QueryBuilder::new()
            .optional("statistics", self.statistics.then_some("true"))
            .optional_encoded("search", self.search.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/groups/{}/subgroups{}", group, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all group tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListGroups>();
    registry.register::<GetGroup>();
    registry.register::<ListGroupMembers>();
    registry.register::<ListGroupProjects>();
    registry.register::<ListSubgroups>();
}
