//! Label tools
//!
//! Tools for managing project and group labels.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List project labels
#[gitlab_tool(
    name = "list_labels",
    description = "List labels in a project",
    category = "labels",
    operation = "read"
)]
pub struct ListLabels {
    /// Project path or ID
    pub project: String,
    /// Include labels from ancestor groups
    #[serde(default)]
    pub include_ancestor_groups: bool,
    /// Search for labels matching this string
    #[serde(default)]
    pub search: Option<String>,
    /// Number of labels per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListLabels {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let mut params = Vec::new();

        if self.include_ancestor_groups {
            params.push("include_ancestor_groups=true".to_string());
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

        let endpoint = format!("/projects/{}/labels{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a specific label
#[gitlab_tool(
    name = "get_label",
    description = "Get details of a specific label",
    category = "labels",
    operation = "read"
)]
pub struct GetLabel {
    /// Project path or ID
    pub project: String,
    /// Label ID or name
    pub label_id: String,
}

#[async_trait]
impl ToolExecutor for GetLabel {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let label_id = urlencoding::encode(&self.label_id);
        let endpoint = format!("/projects/{}/labels/{}", project, label_id);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Create a new label
#[gitlab_tool(
    name = "create_label",
    description = "Create a new label in a project",
    category = "labels",
    operation = "write"
)]
pub struct CreateLabel {
    /// Project path or ID
    pub project: String,
    /// Label name
    pub name: String,
    /// Label color (hex code with #, e.g., "#FF0000")
    pub color: String,
    /// Label description
    #[serde(default)]
    pub description: Option<String>,
    /// Priority for label lists (lower = higher priority)
    #[serde(default)]
    pub priority: Option<u32>,
}

#[async_trait]
impl ToolExecutor for CreateLabel {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/labels", project);

        let mut body = serde_json::json!({
            "name": self.name,
            "color": self.color,
        });

        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(priority) = self.priority {
            body["priority"] = serde_json::Value::Number(priority.into());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Update a label
#[gitlab_tool(
    name = "update_label",
    description = "Update an existing label",
    category = "labels",
    operation = "write"
)]
pub struct UpdateLabel {
    /// Project path or ID
    pub project: String,
    /// Label ID or name
    pub label_id: String,
    /// New label name
    #[serde(default)]
    pub new_name: Option<String>,
    /// New label color (hex code with #)
    #[serde(default)]
    pub color: Option<String>,
    /// New description
    #[serde(default)]
    pub description: Option<String>,
    /// New priority
    #[serde(default)]
    pub priority: Option<u32>,
}

#[async_trait]
impl ToolExecutor for UpdateLabel {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let label_id = urlencoding::encode(&self.label_id);
        let endpoint = format!("/projects/{}/labels/{}", project, label_id);

        let mut body = serde_json::json!({});

        if let Some(ref new_name) = self.new_name {
            body["new_name"] = serde_json::Value::String(new_name.clone());
        }
        if let Some(ref color) = self.color {
            body["color"] = serde_json::Value::String(color.clone());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(priority) = self.priority {
            body["priority"] = serde_json::Value::Number(priority.into());
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a label
#[gitlab_tool(
    name = "delete_label",
    description = "Delete a label from a project",
    category = "labels",
    operation = "delete"
)]
pub struct DeleteLabel {
    /// Project path or ID
    pub project: String,
    /// Label ID or name
    pub label_id: String,
}

#[async_trait]
impl ToolExecutor for DeleteLabel {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let label_id = urlencoding::encode(&self.label_id);
        let endpoint = format!("/projects/{}/labels/{}", project, label_id);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Label '{}' deleted successfully",
            self.label_id
        )))
    }
}
