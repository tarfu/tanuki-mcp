//! Group tools
//!
//! Tools for managing GitLab groups.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
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
        let mut params = Vec::new();

        if let Some(ref search) = self.search {
            params.push(format!("search={}", urlencoding::encode(search)));
        }
        if self.owned {
            params.push("owned=true".to_string());
        }
        if let Some(ref visibility) = self.visibility {
            params.push(format!("visibility={}", visibility));
        }
        if self.statistics {
            params.push("statistics=true".to_string());
        }
        if let Some(ref order_by) = self.order_by {
            params.push(format!("order_by={}", order_by));
        }
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={}", sort));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

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
        let mut params = Vec::new();

        if self.statistics {
            params.push("statistics=true".to_string());
        }
        if self.with_projects {
            params.push("with_projects=true".to_string());
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

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
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(format!("query={}", urlencoding::encode(query)));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let endpoint = format!("/groups/{}/members{}", group, query);
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
        let mut params = Vec::new();

        if self.include_subgroups {
            params.push("include_subgroups=true".to_string());
        }
        if let Some(archived) = self.archived {
            params.push(format!("archived={}", archived));
        }
        if let Some(ref visibility) = self.visibility {
            params.push(format!("visibility={}", visibility));
        }
        if let Some(ref search) = self.search {
            params.push(format!("search={}", urlencoding::encode(search)));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

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
        let mut params = Vec::new();

        if self.statistics {
            params.push("statistics=true".to_string());
        }
        if let Some(ref search) = self.search {
            params.push(format!("search={}", urlencoding::encode(search)));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

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
