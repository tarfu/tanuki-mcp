//! Merge Request Discussion tools
//!
//! Tools for managing discussions and threads on merge requests.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;
use tanuki_mcp_macros::gitlab_tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// List MR discussions
#[gitlab_tool(
    name = "list_mr_discussions",
    description = "List all discussions on a merge request",
    category = "mr_discussions",
    operation = "read"
)]
pub struct ListMrDiscussions {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Number of discussions per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListMrDiscussions {
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

        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions{}",
            project, self.merge_request_iid, query
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a specific discussion
#[gitlab_tool(
    name = "get_mr_discussion",
    description = "Get a specific discussion on a merge request",
    category = "mr_discussions",
    operation = "read"
)]
pub struct GetMrDiscussion {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion ID
    pub discussion_id: String,
}

#[async_trait]
impl ToolExecutor for GetMrDiscussion {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions/{}",
            project, self.merge_request_iid, self.discussion_id
        );

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Position for diff note
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiffPosition {
    /// Base commit SHA
    pub base_sha: String,
    /// Start commit SHA
    pub start_sha: String,
    /// Head commit SHA
    pub head_sha: String,
    /// Position type: text or image
    pub position_type: String,
    /// File path for old version
    #[serde(default)]
    pub old_path: Option<String>,
    /// File path for new version
    #[serde(default)]
    pub new_path: Option<String>,
    /// Line number in old version
    #[serde(default)]
    pub old_line: Option<u32>,
    /// Line number in new version
    #[serde(default)]
    pub new_line: Option<u32>,
}

/// Create a new discussion
#[gitlab_tool(
    name = "create_mr_discussion",
    description = "Create a new discussion thread on a merge request",
    category = "mr_discussions",
    operation = "write"
)]
pub struct CreateMrDiscussion {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion body (Markdown supported)
    pub body: String,
    /// Commit SHA to associate with (for commit comments)
    #[serde(default)]
    pub commit_id: Option<String>,
    /// Position for diff comments
    #[serde(default)]
    pub position: Option<DiffPosition>,
}

#[async_trait]
impl ToolExecutor for CreateMrDiscussion {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions",
            project, self.merge_request_iid
        );

        let mut body = serde_json::json!({
            "body": self.body,
        });

        if let Some(ref commit_id) = self.commit_id {
            body["commit_id"] = serde_json::Value::String(commit_id.clone());
        }
        if let Some(ref position) = self.position {
            body["position"] = serde_json::to_value(position).unwrap_or_default();
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Add note to existing discussion
#[gitlab_tool(
    name = "add_mr_discussion_note",
    description = "Add a note to an existing discussion thread",
    category = "mr_discussions",
    operation = "write"
)]
pub struct AddMrDiscussionNote {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion ID
    pub discussion_id: String,
    /// Note body
    pub body: String,
}

#[async_trait]
impl ToolExecutor for AddMrDiscussionNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions/{}/notes",
            project, self.merge_request_iid, self.discussion_id
        );

        let body = serde_json::json!({
            "body": self.body,
        });

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Update a discussion note
#[gitlab_tool(
    name = "update_mr_discussion_note",
    description = "Update a note in a discussion",
    category = "mr_discussions",
    operation = "write"
)]
pub struct UpdateMrDiscussionNote {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion ID
    pub discussion_id: String,
    /// Note ID
    pub note_id: u64,
    /// New note body
    pub body: String,
}

#[async_trait]
impl ToolExecutor for UpdateMrDiscussionNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions/{}/notes/{}",
            project, self.merge_request_iid, self.discussion_id, self.note_id
        );

        let body = serde_json::json!({
            "body": self.body,
        });

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Delete a discussion note
#[gitlab_tool(
    name = "delete_mr_discussion_note",
    description = "Delete a note from a discussion",
    category = "mr_discussions",
    operation = "delete"
)]
pub struct DeleteMrDiscussionNote {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion ID
    pub discussion_id: String,
    /// Note ID
    pub note_id: u64,
}

#[async_trait]
impl ToolExecutor for DeleteMrDiscussionNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions/{}/notes/{}",
            project, self.merge_request_iid, self.discussion_id, self.note_id
        );

        ctx.gitlab.delete(&endpoint).await?;
        Ok(ToolOutput::text("Discussion note deleted successfully"))
    }
}

/// Resolve a discussion thread
#[gitlab_tool(
    name = "resolve_mr_discussion",
    description = "Resolve or unresolve a discussion thread",
    category = "mr_discussions",
    operation = "write"
)]
pub struct ResolveMrDiscussion {
    /// Project path or ID
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Discussion ID
    pub discussion_id: String,
    /// Resolve (true) or unresolve (false)
    pub resolved: bool,
}

#[async_trait]
impl ToolExecutor for ResolveMrDiscussion {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/discussions/{}",
            project, self.merge_request_iid, self.discussion_id
        );

        let body = serde_json::json!({
            "resolved": self.resolved,
        });

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all MR discussion tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListMrDiscussions>();
    registry.register::<GetMrDiscussion>();
    registry.register::<CreateMrDiscussion>();
    registry.register::<AddMrDiscussionNote>();
    registry.register::<UpdateMrDiscussionNote>();
    registry.register::<DeleteMrDiscussionNote>();
    registry.register::<ResolveMrDiscussion>();
}
