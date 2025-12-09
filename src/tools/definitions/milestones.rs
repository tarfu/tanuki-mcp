//! Milestone tools
//!
//! Tools for managing project milestones.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List project milestones
#[gitlab_tool(
    name = "list_milestones",
    description = "List milestones in a project",
    category = "milestones",
    operation = "read"
)]
pub struct ListMilestones {
    /// Project path or ID
    pub project: String,
    /// Filter by state: active, closed, or all
    #[serde(default)]
    pub state: Option<String>,
    /// Search by title
    #[serde(default)]
    pub search: Option<String>,
    /// Include milestones from parent groups
    #[serde(default)]
    pub include_parent_milestones: bool,
    /// Number of milestones per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListMilestones {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("state", self.state.as_ref())
            .optional_encoded("search", self.search.as_ref())
            .optional(
                "include_parent_milestones",
                self.include_parent_milestones.then_some("true"),
            )
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/milestones{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific milestone
#[gitlab_tool(
    name = "get_milestone",
    description = "Get details of a specific milestone",
    category = "milestones",
    operation = "read"
)]
pub struct GetMilestone {
    /// Project path or ID
    pub project: String,
    /// Milestone ID
    pub milestone_id: u64,
}

#[async_trait]
impl ToolExecutor for GetMilestone {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/milestones/{}", project, self.milestone_id);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Create a new milestone
#[gitlab_tool(
    name = "create_milestone",
    description = "Create a new milestone in a project",
    category = "milestones",
    operation = "write"
)]
pub struct CreateMilestone {
    /// Project path or ID
    pub project: String,
    /// Milestone title
    pub title: String,
    /// Milestone description
    #[serde(default)]
    pub description: Option<String>,
    /// Due date (YYYY-MM-DD format)
    #[serde(default)]
    pub due_date: Option<String>,
    /// Start date (YYYY-MM-DD format)
    #[serde(default)]
    pub start_date: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateMilestone {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/milestones", project);

        let mut body = serde_json::json!({
            "title": self.title,
        });

        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref due_date) = self.due_date {
            body["due_date"] = serde_json::Value::String(due_date.clone());
        }
        if let Some(ref start_date) = self.start_date {
            body["start_date"] = serde_json::Value::String(start_date.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Update a milestone
#[gitlab_tool(
    name = "update_milestone",
    description = "Update an existing milestone",
    category = "milestones",
    operation = "write"
)]
pub struct UpdateMilestone {
    /// Project path or ID
    pub project: String,
    /// Milestone ID
    pub milestone_id: u64,
    /// New title
    #[serde(default)]
    pub title: Option<String>,
    /// New description
    #[serde(default)]
    pub description: Option<String>,
    /// New due date (YYYY-MM-DD format)
    #[serde(default)]
    pub due_date: Option<String>,
    /// New start date (YYYY-MM-DD format)
    #[serde(default)]
    pub start_date: Option<String>,
    /// New state: close or activate
    #[serde(default)]
    pub state_event: Option<String>,
}

#[async_trait]
impl ToolExecutor for UpdateMilestone {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/milestones/{}", project, self.milestone_id);

        let mut body = serde_json::json!({});

        if let Some(ref title) = self.title {
            body["title"] = serde_json::Value::String(title.clone());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref due_date) = self.due_date {
            body["due_date"] = serde_json::Value::String(due_date.clone());
        }
        if let Some(ref start_date) = self.start_date {
            body["start_date"] = serde_json::Value::String(start_date.clone());
        }
        if let Some(ref state_event) = self.state_event {
            body["state_event"] = serde_json::Value::String(state_event.clone());
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a milestone
#[gitlab_tool(
    name = "delete_milestone",
    description = "Delete a milestone from a project",
    category = "milestones",
    operation = "delete"
)]
pub struct DeleteMilestone {
    /// Project path or ID
    pub project: String,
    /// Milestone ID
    pub milestone_id: u64,
}

#[async_trait]
impl ToolExecutor for DeleteMilestone {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/milestones/{}", project, self.milestone_id);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Milestone {} deleted successfully",
            self.milestone_id
        )))
    }
}

/// Get milestone issues
#[gitlab_tool(
    name = "get_milestone_issues",
    description = "Get issues assigned to a milestone",
    category = "milestones",
    operation = "read"
)]
pub struct GetMilestoneIssues {
    /// Project path or ID
    pub project: String,
    /// Milestone ID
    pub milestone_id: u64,
    /// Number of issues per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for GetMilestoneIssues {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!(
            "/projects/{}/milestones/{}/issues{}",
            project, self.milestone_id, query
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get milestone merge requests
#[gitlab_tool(
    name = "get_milestone_merge_requests",
    description = "Get merge requests assigned to a milestone",
    category = "milestones",
    operation = "read"
)]
pub struct GetMilestoneMergeRequests {
    /// Project path or ID
    pub project: String,
    /// Milestone ID
    pub milestone_id: u64,
    /// Number of MRs per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for GetMilestoneMergeRequests {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!(
            "/projects/{}/milestones/{}/merge_requests{}",
            project, self.milestone_id, query
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all milestone tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListMilestones>();
    registry.register::<GetMilestone>();
    registry.register::<CreateMilestone>();
    registry.register::<UpdateMilestone>();
    registry.register::<DeleteMilestone>();
    registry.register::<GetMilestoneIssues>();
    registry.register::<GetMilestoneMergeRequests>();
}
