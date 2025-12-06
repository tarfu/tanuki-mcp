//! User tools
//!
//! Tools for managing users and current user information.

use crate::error::ToolError;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
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
        let mut params = Vec::new();

        if let Some(ref search) = self.search {
            params.push(format!("search={}", urlencoding::encode(search)));
        }
        if let Some(ref username) = self.username {
            params.push(format!("username={}", urlencoding::encode(username)));
        }
        if let Some(active) = self.active {
            params.push(format!("active={}", active));
        }
        if let Some(blocked) = self.blocked {
            params.push(format!("blocked={}", blocked));
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
        let mut params = Vec::new();

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

        let endpoint = format!("/users/{}/events{}", self.user_id, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Register all user tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<GetCurrentUser>();
    registry.register::<ListUsers>();
    registry.register::<GetUser>();
    registry.register::<GetUserActivities>();
}
