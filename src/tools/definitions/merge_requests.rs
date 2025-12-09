//! Merge Request tools
//!
//! Tools for managing GitLab merge requests.

use crate::access_control::{AccessControlled, OperationType, ToolCategory};
use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::{ToolContext, ToolExecutor, ToolInfo, ToolOutput, ToolRegistry};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Register all merge request tools
pub fn register(registry: &mut ToolRegistry) {
    registry.register::<ListMergeRequests>();
    registry.register::<GetMergeRequest>();
    registry.register::<CreateMergeRequest>();
    registry.register::<UpdateMergeRequest>();
    registry.register::<MergeMergeRequest>();
    registry.register::<GetMergeRequestDiffs>();
}

// ============================================================================
// list_merge_requests
// ============================================================================

/// List merge requests in a project
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListMergeRequests {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Filter by state: opened, closed, merged, or all
    #[serde(default)]
    pub state: Option<String>,

    /// Filter by source branch
    #[serde(default)]
    pub source_branch: Option<String>,

    /// Filter by target branch
    #[serde(default)]
    pub target_branch: Option<String>,

    /// Filter by labels (comma-separated)
    #[serde(default)]
    pub labels: Option<String>,

    /// Filter by milestone title
    #[serde(default)]
    pub milestone: Option<String>,

    /// Filter by author ID
    #[serde(default)]
    pub author_id: Option<u64>,

    /// Filter by assignee ID
    #[serde(default)]
    pub assignee_id: Option<u64>,

    /// Search in title and description
    #[serde(default)]
    pub search: Option<String>,

    /// Page number
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

impl ToolInfo for ListMergeRequests {
    fn name() -> &'static str {
        "list_merge_requests"
    }
    fn description() -> &'static str {
        "List merge requests in a GitLab project with optional filtering"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Read
    }
}

impl AccessControlled for ListMergeRequests {
    fn tool_name(&self) -> &'static str {
        "list_merge_requests"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Read
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for ListMergeRequests {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("page", self.page)
            .param("per_page", self.per_page.min(100))
            .optional("state", self.state.as_ref())
            .optional_encoded("source_branch", self.source_branch.as_ref())
            .optional_encoded("target_branch", self.target_branch.as_ref())
            .optional_encoded("labels", self.labels.as_ref())
            .optional_encoded("milestone", self.milestone.as_ref())
            .optional("author_id", self.author_id)
            .optional("assignee_id", self.assignee_id)
            .optional_encoded("search", self.search.as_ref())
            .build();

        let endpoint = format!("/projects/{}/merge_requests{}", project, query);
        let response: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// get_merge_request
// ============================================================================

/// Get details of a specific merge request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetMergeRequest {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Merge request IID
    pub merge_request_iid: u64,

    /// Include commits in response
    #[serde(default)]
    pub include_commits: Option<bool>,

    /// Include changes/diffs in response
    #[serde(default)]
    pub include_changes: Option<bool>,
}

impl ToolInfo for GetMergeRequest {
    fn name() -> &'static str {
        "get_merge_request"
    }
    fn description() -> &'static str {
        "Get detailed information about a specific merge request"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Read
    }
}

impl AccessControlled for GetMergeRequest {
    fn tool_name(&self) -> &'static str {
        "get_merge_request"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Read
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for GetMergeRequest {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);

        // Choose endpoint based on what data is requested
        let endpoint = if self.include_changes == Some(true) {
            format!(
                "/projects/{}/merge_requests/{}/changes",
                project, self.merge_request_iid
            )
        } else {
            format!(
                "/projects/{}/merge_requests/{}",
                project, self.merge_request_iid
            )
        };

        let response: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// create_merge_request
// ============================================================================

/// Create a new merge request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateMergeRequest {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Source branch name
    pub source_branch: String,

    /// Target branch name
    pub target_branch: String,

    /// MR title
    pub title: String,

    /// MR description (Markdown)
    #[serde(default)]
    pub description: Option<String>,

    /// Assignee user IDs
    #[serde(default)]
    pub assignee_ids: Option<Vec<u64>>,

    /// Reviewer user IDs
    #[serde(default)]
    pub reviewer_ids: Option<Vec<u64>>,

    /// Labels (comma-separated)
    #[serde(default)]
    pub labels: Option<String>,

    /// Milestone ID
    #[serde(default)]
    pub milestone_id: Option<u64>,

    /// Create as draft/WIP
    #[serde(default)]
    pub draft: Option<bool>,

    /// Allow commits from upstream members
    #[serde(default)]
    pub allow_collaboration: Option<bool>,

    /// Squash commits when merging
    #[serde(default)]
    pub squash: Option<bool>,

    /// Delete source branch after merge
    #[serde(default)]
    pub remove_source_branch: Option<bool>,
}

impl ToolInfo for CreateMergeRequest {
    fn name() -> &'static str {
        "create_merge_request"
    }
    fn description() -> &'static str {
        "Create a new merge request in a GitLab project"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Write
    }
}

impl AccessControlled for CreateMergeRequest {
    fn tool_name(&self) -> &'static str {
        "create_merge_request"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Write
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for CreateMergeRequest {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/merge_requests", project);

        #[derive(Serialize)]
        struct CreateMRRequest<'a> {
            source_branch: &'a str,
            target_branch: &'a str,
            title: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            assignee_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reviewer_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            milestone_id: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            draft: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            allow_collaboration: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            squash: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove_source_branch: Option<bool>,
        }

        let body = CreateMRRequest {
            source_branch: &self.source_branch,
            target_branch: &self.target_branch,
            title: &self.title,
            description: self.description.as_deref(),
            assignee_ids: self.assignee_ids.as_deref(),
            reviewer_ids: self.reviewer_ids.as_deref(),
            labels: self.labels.as_deref(),
            milestone_id: self.milestone_id,
            draft: self.draft,
            allow_collaboration: self.allow_collaboration,
            squash: self.squash,
            remove_source_branch: self.remove_source_branch,
        };

        let response: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// update_merge_request
// ============================================================================

/// Update an existing merge request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateMergeRequest {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Merge request IID
    pub merge_request_iid: u64,

    /// New title
    #[serde(default)]
    pub title: Option<String>,

    /// New description
    #[serde(default)]
    pub description: Option<String>,

    /// New target branch
    #[serde(default)]
    pub target_branch: Option<String>,

    /// State event: close or reopen
    #[serde(default)]
    pub state_event: Option<String>,

    /// New assignee IDs
    #[serde(default)]
    pub assignee_ids: Option<Vec<u64>>,

    /// New reviewer IDs
    #[serde(default)]
    pub reviewer_ids: Option<Vec<u64>>,

    /// New labels
    #[serde(default)]
    pub labels: Option<String>,

    /// New milestone ID
    #[serde(default)]
    pub milestone_id: Option<u64>,

    /// Set draft status
    #[serde(default)]
    pub draft: Option<bool>,

    /// Set squash on merge
    #[serde(default)]
    pub squash: Option<bool>,

    /// Set remove source branch on merge
    #[serde(default)]
    pub remove_source_branch: Option<bool>,
}

impl ToolInfo for UpdateMergeRequest {
    fn name() -> &'static str {
        "update_merge_request"
    }
    fn description() -> &'static str {
        "Update an existing merge request's properties"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Write
    }
}

impl AccessControlled for UpdateMergeRequest {
    fn tool_name(&self) -> &'static str {
        "update_merge_request"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Write
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for UpdateMergeRequest {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}",
            project, self.merge_request_iid
        );

        #[derive(Serialize)]
        struct UpdateMRRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            target_branch: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            state_event: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            assignee_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reviewer_ids: Option<&'a [u64]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            milestone_id: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            draft: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            squash: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove_source_branch: Option<bool>,
        }

        let body = UpdateMRRequest {
            title: self.title.as_deref(),
            description: self.description.as_deref(),
            target_branch: self.target_branch.as_deref(),
            state_event: self.state_event.as_deref(),
            assignee_ids: self.assignee_ids.as_deref(),
            reviewer_ids: self.reviewer_ids.as_deref(),
            labels: self.labels.as_deref(),
            milestone_id: self.milestone_id,
            draft: self.draft,
            squash: self.squash,
            remove_source_branch: self.remove_source_branch,
        };

        let response: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// merge_merge_request
// ============================================================================

/// Merge a merge request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MergeMergeRequest {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Merge request IID
    pub merge_request_iid: u64,

    /// Custom merge commit message
    #[serde(default)]
    pub merge_commit_message: Option<String>,

    /// Custom squash commit message
    #[serde(default)]
    pub squash_commit_message: Option<String>,

    /// Squash commits
    #[serde(default)]
    pub squash: Option<bool>,

    /// Remove source branch after merge
    #[serde(default)]
    pub should_remove_source_branch: Option<bool>,

    /// Merge when pipeline succeeds
    #[serde(default)]
    pub merge_when_pipeline_succeeds: Option<bool>,

    /// SHA that must match HEAD of source branch
    #[serde(default)]
    pub sha: Option<String>,
}

impl ToolInfo for MergeMergeRequest {
    fn name() -> &'static str {
        "merge_merge_request"
    }
    fn description() -> &'static str {
        "Merge a merge request (requires appropriate permissions)"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Execute
    }
}

impl AccessControlled for MergeMergeRequest {
    fn tool_name(&self) -> &'static str {
        "merge_merge_request"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Execute
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for MergeMergeRequest {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/merge_requests/{}/merge",
            project, self.merge_request_iid
        );

        #[derive(Serialize)]
        struct MergeMRRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            merge_commit_message: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            squash_commit_message: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            squash: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            should_remove_source_branch: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            merge_when_pipeline_succeeds: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            sha: Option<&'a str>,
        }

        let body = MergeMRRequest {
            merge_commit_message: self.merge_commit_message.as_deref(),
            squash_commit_message: self.squash_commit_message.as_deref(),
            squash: self.squash,
            should_remove_source_branch: self.should_remove_source_branch,
            merge_when_pipeline_succeeds: self.merge_when_pipeline_succeeds,
            sha: self.sha.as_deref(),
        };

        let response: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(response)
    }
}

// ============================================================================
// get_merge_request_diffs
// ============================================================================

/// Get diffs for a merge request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetMergeRequestDiffs {
    /// Project ID or URL-encoded path
    pub project: String,

    /// Merge request IID
    pub merge_request_iid: u64,

    /// Page number
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,

    /// Return diffs in unified format
    #[serde(default)]
    pub unidiff: Option<bool>,
}

impl ToolInfo for GetMergeRequestDiffs {
    fn name() -> &'static str {
        "get_merge_request_diffs"
    }
    fn description() -> &'static str {
        "Get the diffs/changes for a merge request"
    }
    fn category() -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type() -> OperationType {
        OperationType::Read
    }
}

impl AccessControlled for GetMergeRequestDiffs {
    fn tool_name(&self) -> &'static str {
        "get_merge_request_diffs"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::MergeRequests
    }
    fn operation_type(&self) -> OperationType {
        OperationType::Read
    }
    fn extract_project(&self) -> Option<String> {
        Some(self.project.clone())
    }
}

#[async_trait]
impl ToolExecutor for GetMergeRequestDiffs {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .param("page", self.page)
            .param("per_page", self.per_page.min(100))
            .optional("unidiff", self.unidiff.filter(|&b| b).map(|_| "true"))
            .build();

        let endpoint = format!(
            "/projects/{}/merge_requests/{}/diffs{}",
            project, self.merge_request_iid, query
        );
        let response: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(response)
    }
}
