//! Wiki tools
//!
//! Tools for managing project wikis.

use crate::error::ToolError;
use crate::gitlab::GitLabClient;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;
use tanuki_mcp_macros::gitlab_tool;

/// List wiki pages
#[gitlab_tool(
    name = "list_wiki_pages",
    description = "List all wiki pages in a project",
    category = "wiki",
    operation = "read"
)]
pub struct ListWikiPages {
    /// Project path or ID
    pub project: String,
    /// Include page content
    #[serde(default)]
    pub with_content: bool,
}

#[async_trait]
impl ToolExecutor for ListWikiPages {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let mut endpoint = format!("/projects/{}/wikis", project);

        if self.with_content {
            endpoint.push_str("?with_content=1");
        }

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Get a wiki page
#[gitlab_tool(
    name = "get_wiki_page",
    description = "Get a specific wiki page",
    category = "wiki",
    operation = "read"
)]
pub struct GetWikiPage {
    /// Project path or ID
    pub project: String,
    /// Wiki page slug (URL-encoded title)
    pub slug: String,
    /// Render the page content as HTML
    #[serde(default)]
    pub render_html: bool,
    /// Get a specific version
    #[serde(default)]
    pub version: Option<String>,
}

#[async_trait]
impl ToolExecutor for GetWikiPage {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let slug = urlencoding::encode(&self.slug);
        let mut params = Vec::new();

        if self.render_html {
            params.push("render_html=true".to_string());
        }
        if let Some(ref version) = self.version {
            params.push(format!("version={}", urlencoding::encode(version)));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let endpoint = format!("/projects/{}/wikis/{}{}", project, slug, query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Create a wiki page
#[gitlab_tool(
    name = "create_wiki_page",
    description = "Create a new wiki page",
    category = "wiki",
    operation = "write"
)]
pub struct CreateWikiPage {
    /// Project path or ID
    pub project: String,
    /// Page title
    pub title: String,
    /// Page content (Markdown)
    pub content: String,
    /// Format: markdown, rdoc, asciidoc, org
    #[serde(default)]
    pub format: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateWikiPage {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let endpoint = format!("/projects/{}/wikis", project);

        let mut body = serde_json::json!({
            "title": self.title,
            "content": self.content,
        });

        if let Some(ref format) = self.format {
            body["format"] = serde_json::Value::String(format.clone());
        }

        let result: serde_json::Value = ctx.gitlab.post(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Update a wiki page
#[gitlab_tool(
    name = "update_wiki_page",
    description = "Update an existing wiki page",
    category = "wiki",
    operation = "write"
)]
pub struct UpdateWikiPage {
    /// Project path or ID
    pub project: String,
    /// Wiki page slug
    pub slug: String,
    /// New title
    #[serde(default)]
    pub title: Option<String>,
    /// New content
    #[serde(default)]
    pub content: Option<String>,
    /// Format: markdown, rdoc, asciidoc, org
    #[serde(default)]
    pub format: Option<String>,
}

#[async_trait]
impl ToolExecutor for UpdateWikiPage {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let slug = urlencoding::encode(&self.slug);
        let endpoint = format!("/projects/{}/wikis/{}", project, slug);

        let mut body = serde_json::json!({});

        if let Some(ref title) = self.title {
            body["title"] = serde_json::Value::String(title.clone());
        }
        if let Some(ref content) = self.content {
            body["content"] = serde_json::Value::String(content.clone());
        }
        if let Some(ref format) = self.format {
            body["format"] = serde_json::Value::String(format.clone());
        }

        let result: serde_json::Value = ctx.gitlab.put(&endpoint, &body).await?;
        ToolOutput::json_value(result)
    }
}

/// Delete a wiki page
#[gitlab_tool(
    name = "delete_wiki_page",
    description = "Delete a wiki page",
    category = "wiki",
    operation = "delete"
)]
pub struct DeleteWikiPage {
    /// Project path or ID
    pub project: String,
    /// Wiki page slug
    pub slug: String,
}

#[async_trait]
impl ToolExecutor for DeleteWikiPage {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let project = GitLabClient::encode_project(&self.project);
        let slug = urlencoding::encode(&self.slug);
        let endpoint = format!("/projects/{}/wikis/{}", project, slug);

        ctx.gitlab.delete(&endpoint).await?;
        Ok(ToolOutput::text(format!(
            "Wiki page '{}' deleted successfully",
            self.slug
        )))
    }
}

/// Register all wiki tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListWikiPages>();
    registry.register::<GetWikiPage>();
    registry.register::<CreateWikiPage>();
    registry.register::<UpdateWikiPage>();
    registry.register::<DeleteWikiPage>();
}
