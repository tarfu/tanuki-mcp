//! Issue tools
//!
//! Tools for managing GitLab issues.

use crate::access_control::{AccessControlled, OperationType, ToolCategory};
use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::{ToolContext, ToolExecutor, ToolInfo, ToolOutput, ToolRegistry};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Register all issue tools
pub fn register(registry: &mut ToolRegistry) {
    registry.register::<ListIssues>();
    registry.register::<GetIssue>();
    registry.register::<CreateIssue>();
    registry.register::<UpdateIssue>();
    registry.register::<DeleteIssue>();
    registry.register::<CreateIssueNote>();
}

// ============================================================================
// list_issues
// ============================================================================

/// List issues in a project with optional filtering
#[derive(Debug, Clone, Deserialize, JsonSchema)]
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

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

impl ToolInfo for ListIssues {
    fn name() -> &'static str {
        "list_issues"
    }
    fn description() -> &'static str {
        "List issues in a GitLab project with optional filtering by state, labels, milestone, assignee, or search terms"
    }
    fn category() -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type() -> OperationType {
        OperationType::Read
    }
}

impl AccessControlled for ListIssues {
    fn tool_name(&self) -> &'static str {
        "list_issues"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Read
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
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

/// Get details of a specific issue
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID (internal ID within the project)
    pub issue_iid: u64,
}

impl ToolInfo for GetIssue {
    fn name() -> &'static str {
        "get_issue"
    }
    fn description() -> &'static str {
        "Get detailed information about a specific issue by its IID"
    }
    fn category() -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type() -> OperationType {
        OperationType::Read
    }
}

impl AccessControlled for GetIssue {
    fn tool_name(&self) -> &'static str {
        "get_issue"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Read
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
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

/// Create a new issue in a project
#[derive(Debug, Clone, Deserialize, JsonSchema)]
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

impl ToolInfo for CreateIssue {
    fn name() -> &'static str {
        "create_issue"
    }
    fn description() -> &'static str {
        "Create a new issue in a GitLab project"
    }
    fn category() -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type() -> OperationType {
        OperationType::Write
    }
}

impl AccessControlled for CreateIssue {
    fn tool_name(&self) -> &'static str {
        "create_issue"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Write
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
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

/// Update an existing issue
#[derive(Debug, Clone, Deserialize, JsonSchema)]
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

impl ToolInfo for UpdateIssue {
    fn name() -> &'static str {
        "update_issue"
    }
    fn description() -> &'static str {
        "Update an existing issue's title, description, state, labels, or other properties"
    }
    fn category() -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type() -> OperationType {
        OperationType::Write
    }
}

impl AccessControlled for UpdateIssue {
    fn tool_name(&self) -> &'static str {
        "update_issue"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Write
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
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

/// Delete an issue
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteIssue {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID
    pub issue_iid: u64,
}

impl ToolInfo for DeleteIssue {
    fn name() -> &'static str {
        "delete_issue"
    }
    fn description() -> &'static str {
        "Delete an issue from a project (requires maintainer or owner permissions)"
    }
    fn category() -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type() -> OperationType {
        OperationType::Delete
    }
}

impl AccessControlled for DeleteIssue {
    fn tool_name(&self) -> &'static str {
        "delete_issue"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Issues
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Delete
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
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

// ============================================================================
// create_issue_note
// ============================================================================

/// Add a comment/note to an issue
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateIssueNote {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Issue IID
    pub issue_iid: u64,

    /// Note body (Markdown supported)
    pub body: String,

    /// Whether this is a confidential note
    #[serde(default)]
    pub confidential: Option<bool>,
}

impl ToolInfo for CreateIssueNote {
    fn name() -> &'static str {
        "create_issue_note"
    }
    fn description() -> &'static str {
        "Add a comment/note to an issue"
    }
    fn category() -> ToolCategory {
        ToolCategory::IssueNotes
    }
    fn operation_type() -> OperationType {
        OperationType::Write
    }
}

impl AccessControlled for CreateIssueNote {
    fn tool_name(&self) -> &'static str {
        "create_issue_note"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::IssueNotes
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Write
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for CreateIssueNote {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/issues/{}/notes", project, self.issue_iid);

        #[derive(Serialize)]
        struct CreateNoteRequest<'a> {
            body: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            confidential: Option<bool>,
        }

        let body = CreateNoteRequest {
            body: &self.body,
            confidential: self.confidential,
        };

        let response: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}
