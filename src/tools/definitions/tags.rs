//! Tag tools
//!
//! Tools for managing repository tags and protected tags.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;
use tanuki_mcp_macros::gitlab_tool;

/// List repository tags
#[gitlab_tool(
    name = "list_tags",
    description = "List repository tags with optional filtering and sorting",
    category = "tags",
    operation = "read"
)]
pub struct ListTags {
    /// Project path or ID
    pub project: String,
    /// Order by: name, updated, or version
    #[serde(default)]
    pub order_by: Option<String>,
    /// Sort direction: asc or desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Search for tags matching this string
    #[serde(default)]
    pub search: Option<String>,
    /// Number of tags per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListTags {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let mut params = Vec::new();

        if let Some(ref order_by) = self.order_by {
            params.push(format!("order_by={}", urlencoding::encode(order_by)));
        }
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={}", urlencoding::encode(sort)));
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

        let endpoint = format!("/projects/{}/repository/tags{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a specific tag
#[gitlab_tool(
    name = "get_tag",
    description = "Get information about a specific tag",
    category = "tags",
    operation = "read"
)]
pub struct GetTag {
    /// Project path or ID
    pub project: String,
    /// Tag name
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for GetTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/repository/tags/{}", project, tag_name);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Create a new tag
#[gitlab_tool(
    name = "create_tag",
    description = "Create a new tag pointing to a ref (branch, tag, or commit)",
    category = "tags",
    operation = "write"
)]
pub struct CreateTag {
    /// Project path or ID
    pub project: String,
    /// Name for the new tag
    pub tag_name: String,
    /// Source ref (branch name, tag, or commit SHA)
    pub ref_name: String,
    /// Optional message for annotated tag
    #[serde(default)]
    pub message: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/repository/tags", project);

        let mut body = serde_json::json!({
            "tag_name": self.tag_name,
            "ref": self.ref_name,
        });

        if let Some(ref message) = self.message {
            body["message"] = serde_json::Value::String(message.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a tag
#[gitlab_tool(
    name = "delete_tag",
    description = "Delete a tag from the repository",
    category = "tags",
    operation = "delete"
)]
pub struct DeleteTag {
    /// Project path or ID
    pub project: String,
    /// Tag name to delete
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for DeleteTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/repository/tags/{}", project, tag_name);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Tag '{}' deleted successfully",
            self.tag_name
        )))
    }
}

/// List protected tags
#[gitlab_tool(
    name = "list_protected_tags",
    description = "List protected tags in a project",
    category = "tags",
    operation = "read"
)]
pub struct ListProtectedTags {
    /// Project path or ID
    pub project: String,
    /// Number of items per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListProtectedTags {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
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

        let endpoint = format!("/projects/{}/protected_tags{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a specific protected tag
#[gitlab_tool(
    name = "get_protected_tag",
    description = "Get information about a specific protected tag",
    category = "tags",
    operation = "read"
)]
pub struct GetProtectedTag {
    /// Project path or ID
    pub project: String,
    /// Protected tag name or wildcard pattern
    pub name: String,
}

#[async_trait]
impl ToolExecutor for GetProtectedTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let name = urlencoding::encode(&self.name);
        let endpoint = format!("/projects/{}/protected_tags/{}", project, name);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Protect a tag
#[gitlab_tool(
    name = "protect_tag",
    description = "Protect a tag with access level restrictions",
    category = "tags",
    operation = "write"
)]
pub struct ProtectTag {
    /// Project path or ID
    pub project: String,
    /// Tag name or wildcard pattern (e.g., "v*", "release-*")
    pub name: String,
    /// Create access level: 0 (no one), 30 (developers), 40 (maintainers)
    #[serde(default)]
    pub create_access_level: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ProtectTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/protected_tags", project);

        let mut body = serde_json::json!({
            "name": self.name,
        });

        if let Some(level) = self.create_access_level {
            body["create_access_level"] = serde_json::Value::Number(level.into());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Unprotect a tag
#[gitlab_tool(
    name = "unprotect_tag",
    description = "Remove protection from a tag",
    category = "tags",
    operation = "delete"
)]
pub struct UnprotectTag {
    /// Project path or ID
    pub project: String,
    /// Tag name or wildcard pattern
    pub name: String,
}

#[async_trait]
impl ToolExecutor for UnprotectTag {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let name = urlencoding::encode(&self.name);
        let endpoint = format!("/projects/{}/protected_tags/{}", project, name);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Tag '{}' unprotected successfully",
            self.name
        )))
    }
}

/// Register all tag tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListTags>();
    registry.register::<GetTag>();
    registry.register::<CreateTag>();
    registry.register::<DeleteTag>();
    registry.register::<ListProtectedTags>();
    registry.register::<GetProtectedTag>();
    registry.register::<ProtectTag>();
    registry.register::<UnprotectTag>();
}
