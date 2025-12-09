//! Issue tools
//!
//! Tools for managing GitLab issues.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;
use serde::Serialize;
use tanuki_mcp_macros::gitlab_tool;

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

// ============================================================================
// list_issues
// ============================================================================

/// List issues in a GitLab project with optional filtering by state, labels, milestone, assignee, or search terms
#[gitlab_tool(
    name = "list_issues",
    category = "issues",
    operation = "read",
    project_field = "project"
)]
pub struct ListIssues {
    /// Project ID or URL-encoded path (e.g., "group/project")
    pub project: String,

    /// Filter by state: opened, closed, or all
    #[serde(default)]
    pub state: Option<String>,

    /// Filter by labels (comma-separated)
    #[serde(default)]
    pub labels: Option<String>,

    /// Filter by milestone title
    #[serde(default)]
    pub milestone: Option<String>,

    /// Filter by assignee ID
    #[serde(default)]
    pub assignee_id: Option<u64>,

    /// Filter by author ID
    #[serde(default)]
    pub author_id: Option<u64>,

    /// Search in title and description
    #[serde(default)]
    pub search: Option<String>,

    /// Page number (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page (default: 20, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

#[async_trait]
impl ToolExecutor for ListIssues {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("page", self.page)
            .param("per_page", self.per_page.min(100))
            .optional("state", self.state.as_ref())
            .optional_encoded("labels", self.labels.as_ref())
            .optional_encoded("milestone", self.milestone.as_ref())
            .optional("assignee_id", self.assignee_id)
            .optional("author_id", self.author_id)
            .optional_encoded("search", self.search.as_ref())
            .build();

        let endpoint = format!("/projects/{}/issues{}", project, query);
        let response: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// get_issue
// ============================================================================

/// Get detailed information about a specific issue by its IID
#[gitlab_tool(
    name = "get_issue",
    category = "issues",
    operation = "read",
    project_field = "project"
)]
pub struct GetIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID (internal ID within the project)
    pub issue_iid: u64,
}

#[async_trait]
impl ToolExecutor for GetIssue {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/issues/{}", project, self.issue_iid);

        let response: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// create_issue
// ============================================================================

/// Create a new issue in a GitLab project
#[gitlab_tool(
    name = "create_issue",
    category = "issues",
    operation = "write",
    project_field = "project"
)]
pub struct CreateIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue title
    pub title: String,

    /// Issue description (Markdown supported)
    #[serde(default)]
    pub description: Option<String>,

    /// Comma-separated label names
    #[serde(default)]
    pub labels: Option<String>,

    /// Milestone ID
    #[serde(default)]
    pub milestone_id: Option<u64>,

    /// Assignee user IDs
    #[serde(default)]
    pub assignee_ids: Option<Vec<u64>>,

    /// Due date (YYYY-MM-DD)
    #[serde(default)]
    pub due_date: Option<String>,

    /// Whether the issue is confidential
    #[serde(default)]
    pub confidential: Option<bool>,
}

#[async_trait]
impl ToolExecutor for CreateIssue {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/issues", project);

        #[derive(Serialize)]
        struct CreateIssueRequest<'a> {
            title: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            milestone_id: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            assignee_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            due_date: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            confidential: Option<bool>,
        }

        let body = CreateIssueRequest {
            title: &self.title,
            description: self.description.as_deref(),
            labels: self.labels.as_deref(),
            milestone_id: self.milestone_id,
            assignee_ids: self.assignee_ids.as_deref(),
            due_date: self.due_date.as_deref(),
            confidential: self.confidential,
        };

        let response: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// update_issue
// ============================================================================

/// Update an existing issue's title, description, state, labels, or other properties
#[gitlab_tool(
    name = "update_issue",
    category = "issues",
    operation = "write",
    project_field = "project"
)]
pub struct UpdateIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID
    pub issue_iid: u64,

    /// New title
    #[serde(default)]
    pub title: Option<String>,

    /// New description
    #[serde(default)]
    pub description: Option<String>,

    /// State event: close or reopen
    #[serde(default)]
    pub state_event: Option<String>,

    /// New labels (comma-separated, replaces existing)
    #[serde(default)]
    pub labels: Option<String>,

    /// New milestone ID
    #[serde(default)]
    pub milestone_id: Option<u64>,

    /// New assignee IDs
    #[serde(default)]
    pub assignee_ids: Option<Vec<u64>>,

    /// New due date
    #[serde(default)]
    pub due_date: Option<String>,

    /// Update confidentiality
    #[serde(default)]
    pub confidential: Option<bool>,
}

#[async_trait]
impl ToolExecutor for UpdateIssue {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/issues/{}", project, self.issue_iid);

        #[derive(Serialize)]
        struct UpdateIssueRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            state_event: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            milestone_id: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            assignee_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            due_date: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            confidential: Option<bool>,
        }

        let body = UpdateIssueRequest {
            title: self.title.as_deref(),
            description: self.description.as_deref(),
            state_event: self.state_event.as_deref(),
            labels: self.labels.as_deref(),
            milestone_id: self.milestone_id,
            assignee_ids: self.assignee_ids.as_deref(),
            due_date: self.due_date.as_deref(),
            confidential: self.confidential,
        };

        let response: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// delete_issue
// ============================================================================

/// Delete an issue from a project (requires maintainer or owner permissions)
#[gitlab_tool(
    name = "delete_issue",
    category = "issues",
    operation = "delete",
    project_field = "project"
)]
pub struct DeleteIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID
    pub issue_iid: u64,
}

#[async_trait]
impl ToolExecutor for DeleteIssue {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/issues/{}", project, self.issue_iid);

        ctx.gitlab.delete(&endpoint).await?;
        ToolOutput::json(serde_json::json!({
            "success": true,
            "message": format!("Issue #{} deleted", self.issue_iid)
        }))
    }
}
