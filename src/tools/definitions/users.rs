//! User tools
//!
//! Tools for managing users and current user information.

use crate::error::ToolError;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// Get current user
#[gitlab_tool(
    name = "get_current_user",
    description = "Get information about the currently authenticated user",
    category = "users",
    operation = "read"
)]
pub struct GetCurrentUser {}

#[async_trait]
impl ToolExecutor for GetCurrentUser {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let result: serde_json::Value = ctx.gitlab.get("/user").await?;
        ToolOutput::json_value(result)
    }
}

/// List users
#[gitlab_tool(
    name = "list_users",
    description = "List users (admin only for full list, otherwise limited)",
    category = "users",
    operation = "read"
)]
pub struct ListUsers {
    /// Search by username or name
    #[serde(default)]
    pub search: Option<String>,
    /// Filter by username
    #[serde(default)]
    pub username: Option<String>,
    /// Filter by active/blocked status
    #[serde(default)]
    pub active: Option<bool>,
    /// Filter by blocked status
    #[serde(default)]
    pub blocked: Option<bool>,
    /// Number of users per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListUsers {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = QueryBuilder::new()
            .optional_encoded("search", self.search.as_ref())
            .optional_encoded("username", self.username.as_ref())
            .optional("active", self.active)
            .optional("blocked", self.blocked)
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/users{}", query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific user
#[gitlab_tool(
    name = "get_user",
    description = "Get information about a specific user",
    category = "users",
    operation = "read"
)]
pub struct GetUser {
    /// User ID
    pub user_id: u64,
}

#[async_trait]
impl ToolExecutor for GetUser {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let endpoint = format!("/users/{}", self.user_id);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get user activities
#[gitlab_tool(
    name = "get_user_activities",
    description = "Get recent activities for a user",
    category = "users",
    operation = "read"
)]
pub struct GetUserActivities {
    /// User ID
    pub user_id: u64,
    /// Number of activities per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for GetUserActivities {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = QueryBuilder::new()
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/users/{}/events{}", self.user_id, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}
