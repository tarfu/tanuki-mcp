//! Release tools
//!
//! Tools for managing project releases.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tanuki_mcp_macros::gitlab_tool;

/// List releases
#[gitlab_tool(
    name = "list_releases",
    description = "List releases in a project",
    category = "releases",
    operation = "read"
)]
pub struct ListReleases {
    /// Project path or ID
    pub project: String,
    /// Sort by: released_at or created_at
    #[serde(default)]
    pub order_by: Option<String>,
    /// Sort direction: asc or desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Number of releases per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListReleases {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let mut params = Vec::new();

        if let Some(ref order_by) = self.order_by {
            params.push(format!("order_by={}", order_by));
        }
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={}", sort));
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

        let endpoint = format!("/projects/{}/releases{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a specific release
#[gitlab_tool(
    name = "get_release",
    description = "Get details of a specific release",
    category = "releases",
    operation = "read"
)]
pub struct GetRelease {
    /// Project path or ID
    pub project: String,
    /// Release tag name
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for GetRelease {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/releases/{}", project, tag_name);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Release asset link
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssetLink {
    /// Link name
    pub name: String,
    /// Link URL
    pub url: String,
    /// Link type: other, runbook, image, package
    #[serde(default)]
    pub link_type: Option<String>,
    /// Direct asset path (optional)
    #[serde(default)]
    pub direct_asset_path: Option<String>,
}

/// Create a release
#[gitlab_tool(
    name = "create_release",
    description = "Create a new release",
    category = "releases",
    operation = "write"
)]
pub struct CreateRelease {
    /// Project path or ID
    pub project: String,
    /// Tag name for the release
    pub tag_name: String,
    /// Release name/title
    #[serde(default)]
    pub name: Option<String>,
    /// Release description (supports Markdown)
    #[serde(default)]
    pub description: Option<String>,
    /// Ref (branch/commit) to create the tag from (if tag doesn't exist)
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Released at date (ISO 8601 format)
    #[serde(default)]
    pub released_at: Option<String>,
    /// Milestone titles to associate
    #[serde(default)]
    pub milestones: Option<Vec<String>>,
    /// Asset links
    #[serde(default)]
    pub assets_links: Option<Vec<AssetLink>>,
}

#[async_trait]
impl ToolExecutor for CreateRelease {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/releases", project);

        let mut body = serde_json::json!({
            "tag_name": self.tag_name,
        });

        if let Some(ref name) = self.name {
            body["name"] = serde_json::Value::String(name.clone());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref ref_name) = self.ref_name {
            body["ref"] = serde_json::Value::String(ref_name.clone());
        }
        if let Some(ref released_at) = self.released_at {
            body["released_at"] = serde_json::Value::String(released_at.clone());
        }
        if let Some(ref milestones) = self.milestones {
            body["milestones"] = serde_json::to_value(milestones).unwrap_or_default();
        }
        if let Some(ref links) = self.assets_links {
            body["assets"] = serde_json::json!({
                "links": serde_json::to_value(links).unwrap_or_default()
            });
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Update a release
#[gitlab_tool(
    name = "update_release",
    description = "Update an existing release",
    category = "releases",
    operation = "write"
)]
pub struct UpdateRelease {
    /// Project path or ID
    pub project: String,
    /// Release tag name
    pub tag_name: String,
    /// New release name
    #[serde(default)]
    pub name: Option<String>,
    /// New description
    #[serde(default)]
    pub description: Option<String>,
    /// New released_at date
    #[serde(default)]
    pub released_at: Option<String>,
    /// New milestones
    #[serde(default)]
    pub milestones: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for UpdateRelease {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/releases/{}", project, tag_name);

        let mut body = serde_json::json!({});

        if let Some(ref name) = self.name {
            body["name"] = serde_json::Value::String(name.clone());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref released_at) = self.released_at {
            body["released_at"] = serde_json::Value::String(released_at.clone());
        }
        if let Some(ref milestones) = self.milestones {
            body["milestones"] = serde_json::to_value(milestones).unwrap_or_default();
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Delete a release
#[gitlab_tool(
    name = "delete_release",
    description = "Delete a release (does not delete the associated tag)",
    category = "releases",
    operation = "delete"
)]
pub struct DeleteRelease {
    /// Project path or ID
    pub project: String,
    /// Release tag name
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for DeleteRelease {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/releases/{}", project, tag_name);

        ctx.gitlab.delete(&endpoint).await?;
        Ok(ToolOutput::text(format!(
            "Release '{}' deleted successfully",
            self.tag_name
        )))
    }
}

/// Get release evidence
#[gitlab_tool(
    name = "get_release_evidence",
    description = "Get evidence collection for a release",
    category = "releases",
    operation = "read"
)]
pub struct GetReleaseEvidence {
    /// Project path or ID
    pub project: String,
    /// Release tag name
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for GetReleaseEvidence {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/releases/{}/evidences", project, tag_name);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Collect release evidence
#[gitlab_tool(
    name = "collect_release_evidence",
    description = "Trigger evidence collection for a release (Premium/Ultimate only)",
    category = "releases",
    operation = "write"
)]
pub struct CollectReleaseEvidence {
    /// Project path or ID
    pub project: String,
    /// Release tag name
    pub tag_name: String,
}

#[async_trait]
impl ToolExecutor for CollectReleaseEvidence {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let tag_name = urlencoding::encode(&self.tag_name);
        let endpoint = format!("/projects/{}/releases/{}/evidence", project, tag_name);

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &serde_json::json!({})).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all release tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListReleases>();
    registry.register::<GetRelease>();
    registry.register::<CreateRelease>();
    registry.register::<UpdateRelease>();
    registry.register::<DeleteRelease>();
    registry.register::<GetReleaseEvidence>();
    registry.register::<CollectReleaseEvidence>();
}
