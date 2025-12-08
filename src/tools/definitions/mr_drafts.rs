//! Merge Request draft notes tools

use serde_json::json;
use tanuki_mcp_macros::gitlab_tool;

use crate::error::ToolError;
use crate::tools::{ToolContext, ToolExecutor, ToolOutput, ToolRegistry};
use async_trait::async_trait;

// ============================================================================
// List MR Draft Notes
// ============================================================================

#[gitlab_tool(
    name = "list_mr_draft_notes",
    description = "List all draft notes for a merge request",
    category = "merge_requests",
    operation = "read"
)]
pub struct ListMrDraftNotes {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListMrDraftNotes {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let mut endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes",
            project, self.merge_request_iid
        );

        let mut params = Vec::new();
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
// Get MR Draft Note
// ============================================================================

#[gitlab_tool(
    name = "get_mr_draft_note",
    description = "Get a specific draft note from a merge request",
    category = "merge_requests",
    operation = "read"
)]
pub struct GetMrDraftNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Draft note ID
    pub draft_note_id: u64,
}

#[async_trait]
impl ToolExecutor for GetMrDraftNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes/{}",
            project, self.merge_request_iid, self.draft_note_id
        );

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Create MR Draft Note
// ============================================================================

#[gitlab_tool(
    name = "create_mr_draft_note",
    description = "Create a new draft note on a merge request",
    category = "merge_requests",
    operation = "write"
)]
pub struct CreateMrDraftNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Content of the draft note
    pub note: String,
    /// SHA of the commit to comment on (for inline comments)
    #[serde(default)]
    pub commit_id: Option<String>,
    /// Whether this is an inline diff comment
    #[serde(default)]
    pub in_reply_to_discussion_id: Option<String>,
    /// Whether to resolve the discussion when publishing
    #[serde(default)]
    pub resolve_discussion: Option<bool>,
    /// File path for inline comments
    #[serde(default)]
    pub position_base_sha: Option<String>,
    /// Start SHA for position
    #[serde(default)]
    pub position_start_sha: Option<String>,
    /// Head SHA for position
    #[serde(default)]
    pub position_head_sha: Option<String>,
    /// Position type (text or image)
    #[serde(default)]
    pub position_type: Option<String>,
    /// New file path for position
    #[serde(default)]
    pub position_new_path: Option<String>,
    /// New line number for position
    #[serde(default)]
    pub position_new_line: Option<u32>,
    /// Old file path for position
    #[serde(default)]
    pub position_old_path: Option<String>,
    /// Old line number for position
    #[serde(default)]
    pub position_old_line: Option<u32>,
}

#[async_trait]
impl ToolExecutor for CreateMrDraftNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes",
            project, self.merge_request_iid
        );

        let mut body = json!({
            "note": self.note,
        });

        if let Some(ref commit_id) = self.commit_id {
            body["commit_id"] = json!(commit_id);
        }
        if let Some(ref in_reply_to) = self.in_reply_to_discussion_id {
            body["in_reply_to_discussion_id"] = json!(in_reply_to);
        }
        if let Some(resolve) = self.resolve_discussion {
            body["resolve_discussion"] = json!(resolve);
        }

        // Build position object if any position parameters are provided
        if self.position_base_sha.is_some() || self.position_new_path.is_some() {
            let mut position = json!({});
            if let Some(ref base_sha) = self.position_base_sha {
                position["base_sha"] = json!(base_sha);
            }
            if let Some(ref start_sha) = self.position_start_sha {
                position["start_sha"] = json!(start_sha);
            }
            if let Some(ref head_sha) = self.position_head_sha {
                position["head_sha"] = json!(head_sha);
            }
            if let Some(ref pos_type) = self.position_type {
                position["position_type"] = json!(pos_type);
            }
            if let Some(ref new_path) = self.position_new_path {
                position["new_path"] = json!(new_path);
            }
            if let Some(new_line) = self.position_new_line {
                position["new_line"] = json!(new_line);
            }
            if let Some(ref old_path) = self.position_old_path {
                position["old_path"] = json!(old_path);
            }
            if let Some(old_line) = self.position_old_line {
                position["old_line"] = json!(old_line);
            }
            body["position"] = position;
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Update MR Draft Note
// ============================================================================

#[gitlab_tool(
    name = "update_mr_draft_note",
    description = "Modify an existing draft note",
    category = "merge_requests",
    operation = "write"
)]
pub struct UpdateMrDraftNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Draft note ID
    pub draft_note_id: u64,
    /// New content of the draft note
    #[serde(default)]
    pub note: Option<String>,
    /// Whether to resolve the discussion when publishing
    #[serde(default)]
    pub resolve_discussion: Option<bool>,
    /// New position (for inline comments)
    #[serde(default)]
    pub position: Option<serde_json::Value>,
}

#[async_trait]
impl ToolExecutor for UpdateMrDraftNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes/{}",
            project, self.merge_request_iid, self.draft_note_id
        );

        let mut body = json!({});

        if let Some(ref note) = self.note {
            body["note"] = json!(note);
        }
        if let Some(resolve) = self.resolve_discussion {
            body["resolve_discussion"] = json!(resolve);
        }
        if let Some(ref position) = self.position {
            body["position"] = position.clone();
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

// ============================================================================
// Delete MR Draft Note
// ============================================================================

#[gitlab_tool(
    name = "delete_mr_draft_note",
    description = "Delete a draft note from a merge request",
    category = "merge_requests",
    operation = "delete"
)]
pub struct DeleteMrDraftNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Draft note ID
    pub draft_note_id: u64,
}

#[async_trait]
impl ToolExecutor for DeleteMrDraftNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes/{}",
            project, self.merge_request_iid, self.draft_note_id
        );

        ctx.gitlab.delete(&endpoint).await?;
        ToolOutput::json_value(json!({"status": "deleted", "draft_note_id": self.draft_note_id}))
    }
}

// ============================================================================
// Publish MR Draft Note
// ============================================================================

#[gitlab_tool(
    name = "publish_mr_draft_note",
    description = "Publish a single draft note (make it visible)",
    category = "merge_requests",
    operation = "write"
)]
pub struct PublishMrDraftNote {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
    /// Draft note ID
    pub draft_note_id: u64,
}

#[async_trait]
impl ToolExecutor for PublishMrDraftNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes/{}/publish",
            project, self.merge_request_iid, self.draft_note_id
        );

        let empty_body = json!({});
        ctx.gitlab.put_no_content(&endpoint, &empty_body).await?;
        ToolOutput::json_value(json!({"status": "published", "draft_note_id": self.draft_note_id}))
    }
}

// ============================================================================
// Publish All MR Draft Notes
// ============================================================================

#[gitlab_tool(
    name = "publish_all_mr_draft_notes",
    description = "Publish all draft notes for a merge request at once",
    category = "merge_requests",
    operation = "write"
)]
pub struct PublishAllMrDraftNotes {
    /// Project ID or URL-encoded path
    pub project: String,
    /// Merge request IID
    pub merge_request_iid: u64,
}

#[async_trait]
impl ToolExecutor for PublishAllMrDraftNotes {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = urlencoding::encode(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/draft_notes/bulk_publish",
            project, self.merge_request_iid
        );

        let empty_body = json!({});
        ctx.gitlab.post_no_content(&endpoint, &empty_body).await?;
        ToolOutput::json_value(json!({
            "status": "all_published",
            "merge_request_iid": self.merge_request_iid
        }))
    }
}

/// Register all MR draft notes tools
pub fn register(registry: &mut ToolRegistry) {
    registry.register::<ListMrDraftNotes>();
    registry.register::<GetMrDraftNote>();
    registry.register::<CreateMrDraftNote>();
    registry.register::<UpdateMrDraftNote>();
    registry.register::<DeleteMrDraftNote>();
    registry.register::<PublishMrDraftNote>();
    registry.register::<PublishAllMrDraftNotes>();
}
