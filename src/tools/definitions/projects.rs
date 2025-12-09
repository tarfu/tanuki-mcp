//! Project tools
//!
//! Tools for managing GitLab projects.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use crate::util::QueryBuilder;
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List projects
#[gitlab_tool(
    name = "list_projects",
    description = "List projects accessible to the authenticated user",
    category = "projects",
    operation = "read"
)]
pub struct ListProjects {
    /// Search by project name
    #[serde(default)]
    pub search: Option<String>,
    /// Filter by visibility: public, internal, private
    #[serde(default)]
    pub visibility: Option<String>,
    /// Filter by archived status
    #[serde(default)]
    pub archived: Option<bool>,
    /// Filter by ownership
    #[serde(default)]
    pub owned: bool,
    /// Filter by membership
    #[serde(default)]
    pub membership: bool,
    /// Include project statistics
    #[serde(default)]
    pub statistics: bool,
    /// Sort by: id, name, path, created_at, updated_at, last_activity_at
    #[serde(default)]
    pub order_by: Option<String>,
    /// Sort direction: asc or desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Number of projects per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListProjects {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = QueryBuilder::new()
            .optional_encoded("search", self.search.as_ref())
            .optional("visibility", self.visibility.as_ref())
            .optional("archived", self.archived)
            .optional("owned", self.owned.then_some("true"))
            .optional("membership", self.membership.then_some("true"))
            .optional("statistics", self.statistics.then_some("true"))
            .optional("order_by", self.order_by.as_ref())
            .optional("sort", self.sort.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects{}", query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a specific project
#[gitlab_tool(
    name = "get_project",
    description = "Get details of a specific project",
    category = "projects",
    operation = "read"
)]
pub struct GetProject {
    /// Project path or ID
    pub project: String,
    /// Include project statistics
    #[serde(default)]
    pub statistics: bool,
    /// Include license information
    #[serde(default)]
    pub license: bool,
}

#[async_trait]
impl ToolExecutor for GetProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query = QueryBuilder::new()
            .optional("statistics", self.statistics.then_some("true"))
            .optional("license", self.license.then_some("true"))
            .build();

        let endpoint = format!("/projects/{}{}", project, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Create a new project
#[gitlab_tool(
    name = "create_project",
    description = "Create a new project",
    category = "projects",
    operation = "write"
)]
pub struct CreateProject {
    /// Project name
    pub name: String,
    /// Project path/slug (defaults to name if not provided)
    #[serde(default)]
    pub path: Option<String>,
    /// Namespace ID to create the project in (group or user namespace)
    #[serde(default)]
    pub namespace_id: Option<u64>,
    /// Project description
    #[serde(default)]
    pub description: Option<String>,
    /// Visibility: private, internal, public
    #[serde(default)]
    pub visibility: Option<String>,
    /// Initialize with README
    #[serde(default)]
    pub initialize_with_readme: bool,
    /// Default branch name
    #[serde(default)]
    pub default_branch: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let mut body = serde_json::json!({
            "name": self.name,
        });

        if let Some(ref path) = self.path {
            body["path"] = serde_json::Value::String(path.clone());
        }
        if let Some(namespace_id) = self.namespace_id {
            body["namespace_id"] = serde_json::Value::Number(namespace_id.into());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref visibility) = self.visibility {
            body["visibility"] = serde_json::Value::String(visibility.clone());
        }
        if self.initialize_with_readme {
            body["initialize_with_readme"] = serde_json::Value::Bool(true);
        }
        if let Some(ref default_branch) = self.default_branch {
            body["default_branch"] = serde_json::Value::String(default_branch.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post("/projects", &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Update a project
#[gitlab_tool(
    name = "update_project",
    description = "Update project settings",
    category = "projects",
    operation = "write"
)]
pub struct UpdateProject {
    /// Project path or ID
    pub project: String,
    /// New project name
    #[serde(default)]
    pub name: Option<String>,
    /// New project path
    #[serde(default)]
    pub path: Option<String>,
    /// New description
    #[serde(default)]
    pub description: Option<String>,
    /// New visibility: private, internal, public
    #[serde(default)]
    pub visibility: Option<String>,
    /// New default branch
    #[serde(default)]
    pub default_branch: Option<String>,
    /// Archive the project
    #[serde(default)]
    pub archived: Option<bool>,
}

#[async_trait]
impl ToolExecutor for UpdateProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}", project);

        let mut body = serde_json::json!({});

        if let Some(ref name) = self.name {
            body["name"] = serde_json::Value::String(name.clone());
        }
        if let Some(ref path) = self.path {
            body["path"] = serde_json::Value::String(path.clone());
        }
        if let Some(ref description) = self.description {
            body["description"] = serde_json::Value::String(description.clone());
        }
        if let Some(ref visibility) = self.visibility {
            body["visibility"] = serde_json::Value::String(visibility.clone());
        }
        if let Some(ref default_branch) = self.default_branch {
            body["default_branch"] = serde_json::Value::String(default_branch.clone());
        }
        if let Some(archived) = self.archived {
            body["archived"] = serde_json::Value::Bool(archived);
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// Delete a project
#[gitlab_tool(
    name = "delete_project",
    description = "Delete a project (requires owner permissions)",
    category = "projects",
    operation = "delete"
)]
pub struct DeleteProject {
    /// Project path or ID
    pub project: String,
}

#[async_trait]
impl ToolExecutor for DeleteProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}", project);

        ctx.gitlab.delete(&endpoint).await?;

        Ok(ToolOutput::text(format!(
            "Project '{}' scheduled for deletion",
            self.project
        )))
    }
}

/// Fork a project
#[gitlab_tool(
    name = "fork_project",
    description = "Fork a project to a namespace",
    category = "projects",
    operation = "write"
)]
pub struct ForkProject {
    /// Project path or ID to fork
    pub project: String,
    /// Namespace ID to fork to (defaults to current user's namespace)
    #[serde(default)]
    pub namespace_id: Option<u64>,
    /// Name for the forked project
    #[serde(default)]
    pub name: Option<String>,
    /// Path for the forked project
    #[serde(default)]
    pub path: Option<String>,
}

#[async_trait]
impl ToolExecutor for ForkProject {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/fork", project);

        let mut body = serde_json::json!({});

        if let Some(namespace_id) = self.namespace_id {
            body["namespace_id"] = serde_json::Value::Number(namespace_id.into());
        }
        if let Some(ref name) = self.name {
            body["name"] = serde_json::Value::String(name.clone());
        }
        if let Some(ref path) = self.path {
            body["path"] = serde_json::Value::String(path.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;

        ToolOutput::json_value(result)
    }
}

/// List project members
#[gitlab_tool(
    name = "list_project_members",
    description = "List members of a project",
    category = "projects",
    operation = "read"
)]
pub struct ListProjectMembers {
    /// Project path or ID
    pub project: String,
    /// Search by name or username
    #[serde(default)]
    pub query: Option<String>,
    /// Number of members per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListProjectMembers {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let query_str = QueryBuilder::new()
            .optional_encoded("query", self.query.as_ref())
            .optional("per_page", self.per_page.map(|p| p.min(100)))
            .optional("page", self.page)
            .build();

        let endpoint = format!("/projects/{}/members{}", project, query_str);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Register all project tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListProjects>();
    registry.register::<GetProject>();
    registry.register::<CreateProject>();
    registry.register::<UpdateProject>();
    registry.register::<DeleteProject>();
    registry.register::<ForkProject>();
    registry.register::<ListProjectMembers>();
}
