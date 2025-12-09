//! Issue links tools

use serde_json::json;
use tanuki_mcp_macros::gitlab_tool;

use crate::error::ToolError;
use crate::tools::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;

// ============================================================================
// List Issue Links
// ============================================================================

#[gitlab_tool(
    name = "list_issue_links",
    description = "List all linked issues for a given issue",
    category = "issues",
    operation = "read"
)]
pub struct ListIssueLinks {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
}

#[async_trait]
impl ToolExecutor for ListIssueLinks {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!("/projects/{}/issues/{}/links", project, self.issue_iid);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Create Issue Link
// ============================================================================

#[gitlab_tool(
    name = "create_issue_link",
    description = "Create a link between two issues",
    category = "issues",
    operation = "write"
)]
pub struct CreateIssueLink {
    /// Project ID or URL-encoded path of the source issue
    pub project: String,
    /// Source issue IID
    pub issue_iid: u64,
    /// Project ID or URL-encoded path of the target issue
    pub target_project: String,
    /// Target issue IID
    pub target_issue_iid: u64,
    /// Link type: relates_to, blocks, is_blocked_by
    #[serde(default)]
    pub link_type: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateIssueLink {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!("/projects/{}/issues/{}/links", project, self.issue_iid);

        let mut body = json!({
            "target_project_id": self.target_project,
            "target_issue_iid": self.target_issue_iid,
        });

        if let Some(ref link_type) = self.link_type {
            body["link_type"] = json!(link_type);
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Delete Issue Link
// ============================================================================

#[gitlab_tool(
    name = "delete_issue_link",
    description = "Remove a link between two issues",
    category = "issues",
    operation = "delete"
)]
pub struct DeleteIssueLink {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Issue IID
    pub issue_iid: u64,
    /// Issue link ID (not the target issue IID)
    pub issue_link_id: u64,
}

#[async_trait]
impl ToolExecutor for DeleteIssueLink {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/issues/{}/links/{}",
            project, self.issue_iid, self.issue_link_id
        );

        ctx.gitlab.delete(&endpoint).await?;
        ToolOutput::json_value(json!({
            "status": "deleted",
            "issue_link_id": self.issue_link_id
        }))
    }
}
