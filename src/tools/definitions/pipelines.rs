//! Pipeline tools
//!
//! Tools for managing CI/CD pipelines and jobs.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tanuki_mcp_macros::gitlab_tool;

/// List pipelines in a project
#[gitlab_tool(
    name = "list_pipelines",
    description = "List pipelines in a project with optional filtering",
    category = "pipelines",
    operation = "read"
)]
pub struct ListPipelines {
    /// Project path or ID
    pub project: String,
    /// Filter by status: running, pending, success, failed, canceled, skipped, manual, scheduled
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by ref (branch or tag)
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Filter by SHA
    #[serde(default)]
    pub sha: Option<String>,
    /// Filter by username
    #[serde(default)]
    pub username: Option<String>,
    /// Sort order: asc or desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Number of pipelines per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListPipelines {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("status", self.status.as_ref())
            .optional_encoded("ref", self.ref_name.as_ref())
            .optional("sha", self.sha.as_ref())
            .optional_encoded("username", self.username.as_ref())
            .optional("sort", self.sort.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/pipelines{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific pipeline
#[gitlab_tool(
    name = "get_pipeline",
    description = "Get details of a specific pipeline",
    category = "pipelines",
    operation = "read"
)]
pub struct GetPipeline {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
}

#[async_trait]
impl ToolExecutor for GetPipeline {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/pipelines/{}", project, self.pipeline_id);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Pipeline variable for create/play operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineVariable {
    /// Variable key
    pub key: String,
    /// Variable value
    pub value: String,
    /// Variable type: env_var or file
    #[serde(default)]
    pub variable_type: Option<String>,
}

/// Create a new pipeline
#[gitlab_tool(
    name = "create_pipeline",
    description = "Trigger a new pipeline for a ref (branch or tag)",
    category = "pipelines",
    operation = "write"
)]
pub struct CreatePipeline {
    /// Project path or ID
    pub project: String,
    /// Branch or tag to run the pipeline for
    pub ref_name: String,
    /// Pipeline variables (key-value pairs)
    #[serde(default)]
    pub variables: Option<Vec<PipelineVariable>>,
}

#[async_trait]
impl ToolExecutor for CreatePipeline {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/pipeline", project);

        let mut body = serde_json::json!({
            "ref": self.ref_name,
        });

        if let Some(ref vars) = self.variables {
            body["variables"] = serde_json::to_value(vars).unwrap_or_default();
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Retry a pipeline
#[gitlab_tool(
    name = "retry_pipeline",
    description = "Retry all failed jobs in a pipeline",
    category = "pipelines",
    operation = "write"
)]
pub struct RetryPipeline {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
}

#[async_trait]
impl ToolExecutor for RetryPipeline {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/pipelines/{}/retry", project, self.pipeline_id);

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &serde_json::json!({})).await?;

        ToolOutput::json_value(result)
    }
}

/// Cancel a pipeline
#[gitlab_tool(
    name = "cancel_pipeline",
    description = "Cancel a running pipeline",
    category = "pipelines",
    operation = "write"
)]
pub struct CancelPipeline {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
}

#[async_trait]
impl ToolExecutor for CancelPipeline {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/pipelines/{}/cancel",
            project, self.pipeline_id
        );

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &serde_json::json!({})).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a pipeline
#[gitlab_tool(
    name = "delete_pipeline",
    description = "Delete a pipeline and all its jobs",
    category = "pipelines",
    operation = "delete"
)]
pub struct DeletePipeline {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
}

#[async_trait]
impl ToolExecutor for DeletePipeline {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/pipelines/{}", project, self.pipeline_id);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Pipeline {} deleted successfully",
            self.pipeline_id
        )))
    }
}

/// List jobs in a pipeline
#[gitlab_tool(
    name = "list_pipeline_jobs",
    description = "List all jobs in a pipeline",
    category = "pipelines",
    operation = "read"
)]
pub struct ListPipelineJobs {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
    /// Include retried jobs
    #[serde(default)]
    pub include_retried: bool,
    /// Filter by scope: created, pending, running, failed, success, canceled, skipped, manual
    #[serde(default)]
    pub scope: Option<Vec<String>>,
    /// Number of jobs per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListPipelineJobs {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let mut query = QueryBuilder::new()
            .optional("include_retried", self.include_retried.then_some("true"))
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .build();

        // Handle scope[] array parameters manually since QueryBuilder doesn't support arrays
        if let Some(ref scopes) = self.scope {
            for scope in scopes {
                let sep = if query.is_empty() { "?" } else { "&" };
                query.push_str(&format!("{}scope[]={}", sep, scope));
            }
        }

        let endpoint = format!(
            "/projects/{}/pipelines/{}/jobs{}",
            project, self.pipeline_id, query
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific job
#[gitlab_tool(
    name = "get_job",
    description = "Get details of a specific job",
    category = "pipelines",
    operation = "read"
)]
pub struct GetJob {
    /// Project path or ID
    pub project: String,
    /// Job ID
    pub job_id: u64,
}

#[async_trait]
impl ToolExecutor for GetJob {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/jobs/{}", project, self.job_id);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get job log/trace
#[gitlab_tool(
    name = "get_job_log",
    description = "Get the log (trace) output of a job",
    category = "pipelines",
    operation = "read"
)]
pub struct GetJobLog {
    /// Project path or ID
    pub project: String,
    /// Job ID
    pub job_id: u64,
}

#[async_trait]
impl ToolExecutor for GetJobLog {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/jobs/{}/trace", project, self.job_id);

        // The trace endpoint returns plain text, not JSON
        let result = ctx.gitlab.get_text(&endpoint).await?;

        Ok(ToolOutput::text(result))
    }
}

/// Retry a job
#[gitlab_tool(
    name = "retry_job",
    description = "Retry a failed or canceled job",
    category = "pipelines",
    operation = "write"
)]
pub struct RetryJob {
    /// Project path or ID
    pub project: String,
    /// Job ID
    pub job_id: u64,
}

#[async_trait]
impl ToolExecutor for RetryJob {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/jobs/{}/retry", project, self.job_id);

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &serde_json::json!({})).await?;

        ToolOutput::json_value(result)
    }
}

/// Cancel a job
#[gitlab_tool(
    name = "cancel_job",
    description = "Cancel a running job",
    category = "pipelines",
    operation = "write"
)]
pub struct CancelJob {
    /// Project path or ID
    pub project: String,
    /// Job ID
    pub job_id: u64,
}

#[async_trait]
impl ToolExecutor for CancelJob {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/jobs/{}/cancel", project, self.job_id);

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &serde_json::json!({})).await?;

        ToolOutput::json_value(result)
    }
}

/// Play (trigger) a manual job
#[gitlab_tool(
    name = "play_job",
    description = "Trigger a manual job",
    category = "pipelines",
    operation = "write"
)]
pub struct PlayJob {
    /// Project path or ID
    pub project: String,
    /// Job ID
    pub job_id: u64,
    /// Job variables (key-value pairs)
    #[serde(default)]
    pub job_variables_attributes: Option<Vec<PipelineVariable>>,
}

#[async_trait]
impl ToolExecutor for PlayJob {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/jobs/{}/play", project, self.job_id);

        let mut body = serde_json::json!({});

        if let Some(ref vars) = self.job_variables_attributes {
            body["job_variables_attributes"] = serde_json::to_value(vars).unwrap_or_default();
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Get pipeline variables
#[gitlab_tool(
    name = "get_pipeline_variables",
    description = "Get variables for a pipeline",
    category = "pipelines",
    operation = "read"
)]
pub struct GetPipelineVariables {
    /// Project path or ID
    pub project: String,
    /// Pipeline ID
    pub pipeline_id: u64,
}

#[async_trait]
impl ToolExecutor for GetPipelineVariables {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!(
            "/projects/{}/pipelines/{}/variables",
            project, self.pipeline_id
        );

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}
