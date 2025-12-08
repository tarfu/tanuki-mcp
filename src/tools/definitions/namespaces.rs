//! Namespace tools
//!
//! Tools for listing and searching namespaces (users and groups).

use crate::error::ToolError;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
use async_trait::async_trait;

use tanuki_mcp_macros::gitlab_tool;

/// List namespaces
#[gitlab_tool(
    name = "list_namespaces",
    description = "List all namespaces accessible to the authenticated user",
    category = "namespaces",
    operation = "read"
)]
pub struct ListNamespaces {
    /// Search by namespace name or path
    #[serde(default)]
    pub search: Option<String>,
    /// Filter by owned namespaces only
    #[serde(default)]
    pub owned_only: bool,
    /// Number of namespaces per page (max 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[async_trait]
impl ToolExecutor for ListNamespaces {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let mut params = Vec::new();

        if let Some(ref search) = self.search {
            params.push(format!("search={}", urlencoding::encode(search)));
        }
        if self.owned_only {
            params.push("owned_only=true".to_string());
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

        let endpoint = format!("/namespaces{}", query);
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Get a namespace by ID or path
#[gitlab_tool(
    name = "get_namespace",
    description = "Get details of a specific namespace",
    category = "namespaces",
    operation = "read"
)]
pub struct GetNamespace {
    /// Namespace ID or URL-encoded path
    pub namespace: String,
}

#[async_trait]
impl ToolExecutor for GetNamespace {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let namespace = urlencoding::encode(&self.namespace);
        let endpoint = format!("/namespaces/{}", namespace);

        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;
        ToolOutput::json_value(result)
    }
}

/// Check if a namespace exists
#[gitlab_tool(
    name = "namespace_exists",
    description = "Check if a namespace path exists and is available",
    category = "namespaces",
    operation = "read"
)]
pub struct NamespaceExists {
    /// Namespace path to check
    pub path: String,
    /// Parent namespace ID (optional)
    #[serde(default)]
    pub parent_id: Option<u64>,
}

#[async_trait]
impl ToolExecutor for NamespaceExists {
    async fn execute(&self, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let mut params = vec![format!("path={}", urlencoding::encode(&self.path))];

        if let Some(parent_id) = self.parent_id {
            params.push(format!("parent_id={}", parent_id));
        }

        let endpoint = format!(
            "/namespaces/{}/exists?{}",
            urlencoding::encode(&self.path),
            params.join("&")
        );
        let result: serde_json::Value = ctx.gitlab.get(&endpoint).await?;

        ToolOutput::json_value(result)
    }
}

/// Register all namespace tools
pub fn register(registry: &mut crate::tools::ToolRegistry) {
    registry.register::<ListNamespaces>();
    registry.register::<GetNamespace>();
    registry.register::<NamespaceExists>();
}
