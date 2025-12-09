//! Issue notes (comments) tools

use serde_json::json;
use tanuki_mcp_macros::gitlab_tool;

use crate::error::ToolError;
use crate::tools::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;

// ============================================================================
// List Issue Notes
// ============================================================================

#[gitlab_tool(
    name = "list_issue_notes",
    description = "List all notes (comments) on an issue",
    category = "issues",
    operation = "read"
)]
pub struct ListIssueNotes {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// Sort order (asc or desc)
    #[serde(default)]
    pub sort: Option<String>,
    /// Order by (created_at or updated_at)
    #[serde(default)]
    pub order_by: Option<String>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListIssueNotes {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let mut endpoint = format!("/projects/{}/issues/{}/notes", project, self.issue_iid);

        let mut params = Vec::new();
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={}", sort));
        }
        if let Some(ref order_by) = self.order_by {
            params.push(format!("order_by={}", order_by));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }
        if let Some(per_page) = self.per_page {
            params.push(format!("per_page={}", per_page));
        }

        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Create Issue Note
// ============================================================================

#[gitlab_tool(
    name = "create_issue_note",
    description = "Add a new comment (note) to an issue",
    category = "issues",
    operation = "write"
)]
pub struct CreateIssueNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// The content of the note/comment
    pub body: String,
    /// Whether to create a confidential note (internal)
    #[serde(default)]
    pub confidential: Option<bool>,
    /// Create at a specific time (ISO 8601 format, admin only)
    #[serde(default)]
    pub created_at: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateIssueNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!("/projects/{}/issues/{}/notes", project, self.issue_iid);

        let mut body = json!({
            "body": self.body,
        });

        if let Some(confidential) = self.confidential {
            body["confidential"] = json!(confidential);
        }
        if let Some(ref created_at) = self.created_at {
            body["created_at"] = json!(created_at);
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Get Issue Note
// ============================================================================

#[gitlab_tool(
    name = "get_issue_note",
    description = "Get a specific note from an issue",
    category = "issues",
    operation = "read"
)]
pub struct GetIssueNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// Note ID
    pub note_id: u64,
}

#[async_trait]
impl ToolExecutor for GetIssueNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/issues/{}/notes/{}",
            project, self.issue_iid, self.note_id
        );

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Update Issue Note
// ============================================================================

#[gitlab_tool(
    name = "update_issue_note",
    description = "Modify an existing issue note",
    category = "issues",
    operation = "write"
)]
pub struct UpdateIssueNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// Note ID
    pub note_id: u64,
    /// The new content of the note
    pub body: String,
    /// Whether the note should be confidential
    #[serde(default)]
    pub confidential: Option<bool>,
}

#[async_trait]
impl ToolExecutor for UpdateIssueNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/issues/{}/notes/{}",
            project, self.issue_iid, self.note_id
        );

        let mut body = json!({
            "body": self.body,
        });

        if let Some(confidential) = self.confidential {
            body["confidential"] = json!(confidential);
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Delete Issue Note
// ============================================================================

#[gitlab_tool(
    name = "delete_issue_note",
    description = "Delete a note from an issue",
    category = "issues",
    operation = "delete"
)]
pub struct DeleteIssueNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// Note ID
    pub note_id: u64,
}

#[async_trait]
impl ToolExecutor for DeleteIssueNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/issues/{}/notes/{}",
            project, self.issue_iid, self.note_id
        );

        ctx.gitlab.delete(&endpoint).await?;
        ToolOutput::json_value(json!({"status": "deleted", "note_id": self.note_id}))
    }
}
