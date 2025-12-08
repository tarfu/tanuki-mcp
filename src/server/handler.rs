//! MCP server handler
//!
//! Implements the MCP protocol handler for GitLab tools.

use crate::access_control::AccessResolver;
use crate::config::AppConfig;
use crate::dashboard::DashboardMetrics;
use crate::gitlab::GitLabClient;
use crate::tools::{ContentBlock, ToolContext, ToolOutput, ToolRegistry, definitions};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, Implementation, InitializeResult,
    ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities, Tool,
    ToolsCapability,
};
use rmcp::service::{RequestContext, RoleServer};
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::future::Future;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// GitLab MCP server handler
#[derive(Clone)]
pub struct GitLabMcpHandler {
    /// Server name for MCP
    name: String,
    /// Server version
    version: String,
    /// Tool registry
    registry: Arc<ToolRegistry>,
    /// GitLab client
    gitlab: Arc<GitLabClient>,
    /// Access control resolver
    access: Arc<AccessResolver>,
    /// Dashboard metrics (optional)
    metrics: Option<Arc<DashboardMetrics>>,
}

impl GitLabMcpHandler {
    /// Create a new handler from configuration
    pub fn new(config: &AppConfig, gitlab: GitLabClient, access: AccessResolver) -> Self {
        Self::new_with_shared(config, Arc::new(gitlab), Arc::new(access))
    }

    /// Create a new handler with shared (Arc-wrapped) resources
    ///
    /// This is useful when creating multiple handlers that share the same
    /// GitLab client and access resolver (e.g., for HTTP transport with
    /// multiple concurrent connections).
    pub fn new_with_shared(
        config: &AppConfig,
        gitlab: Arc<GitLabClient>,
        access: Arc<AccessResolver>,
    ) -> Self {
        // Build tool registry
        let mut registry = ToolRegistry::new();
        definitions::register_all_tools(&mut registry);

        info!(tools = registry.len(), "Initialized GitLab MCP handler");

        Self {
            name: config.server.name.clone(),
            version: config.server.version.clone(),
            registry: Arc::new(registry),
            gitlab,
            access,
            metrics: None,
        }
    }

    /// Create a new handler with shared resources and metrics
    pub fn new_with_metrics(
        config: &AppConfig,
        gitlab: Arc<GitLabClient>,
        access: Arc<AccessResolver>,
        metrics: Arc<DashboardMetrics>,
    ) -> Self {
        // Build tool registry
        let mut registry = ToolRegistry::new();
        definitions::register_all_tools(&mut registry);

        info!(
            tools = registry.len(),
            "Initialized GitLab MCP handler with metrics"
        );

        Self {
            name: config.server.name.clone(),
            version: config.server.version.clone(),
            registry: Arc::new(registry),
            gitlab,
            access,
            metrics: Some(metrics),
        }
    }

    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.registry.len()
    }

    /// Create tool context for a request
    fn create_context(&self, request_id: &str) -> ToolContext {
        match &self.metrics {
            Some(metrics) => ToolContext::with_metrics(
                self.gitlab.clone(),
                self.access.clone(),
                request_id,
                metrics.clone(),
            ),
            None => ToolContext::new(self.gitlab.clone(), self.access.clone(), request_id),
        }
    }

    /// Convert internal tool output to MCP result
    fn to_mcp_result(&self, output: ToolOutput) -> CallToolResult {
        let content = output
            .content
            .into_iter()
            .map(|block| match block {
                ContentBlock::Text { text } => Content::text(text),
                ContentBlock::Image { data, mime_type } => {
                    // rmcp supports images via Content::image(data, mime_type)
                    Content::image(data, mime_type)
                }
                ContentBlock::Resource { uri, text, .. } => {
                    // Convert resource to text representation
                    Content::text(text.unwrap_or_else(|| format!("[Resource: {}]", uri)))
                }
            })
            .collect();

        CallToolResult {
            content,
            is_error: Some(output.is_error),
            meta: None,
            structured_content: None,
        }
    }

    /// Convert registry tools to MCP tool definitions
    ///
    /// Tools that are globally denied (denied everywhere, no project grants access)
    /// will have their description prefixed with "UNAVAILABLE: ".
    fn get_mcp_tools(&self) -> Vec<Tool> {
        self.registry
            .tools()
            .map(|tool| {
                // Convert schemars schema to MCP format (JsonObject = Map<String, Value>)
                let schema_value = serde_json::to_value(&tool.input_schema)
                    .unwrap_or_else(|_| serde_json::json!({}));

                // Build the input schema as a JsonObject
                let mut input_schema: Map<String, Value> = Map::new();
                input_schema.insert("type".to_string(), Value::String("object".to_string()));

                if let Some(props) = schema_value.get("properties") {
                    input_schema.insert("properties".to_string(), props.clone());
                }
                if let Some(required) = schema_value.get("required") {
                    input_schema.insert("required".to_string(), required.clone());
                }

                // Check if this tool is globally denied (denied everywhere)
                let is_globally_denied =
                    self.access
                        .is_globally_denied(tool.name, tool.category, tool.operation);

                // Build description - prefix with UNAVAILABLE if globally denied
                let description = if is_globally_denied {
                    format!("UNAVAILABLE: {}", tool.description)
                } else {
                    tool.description.to_string()
                };

                Tool {
                    name: Cow::Owned(tool.name.to_string()),
                    description: Some(Cow::Owned(description)),
                    input_schema: Arc::new(input_schema),
                    annotations: None,
                    icons: None,
                    meta: None,
                    output_schema: None,
                    title: None,
                }
            })
            .collect()
    }

    /// Execute a tool call
    async fn execute_tool(
        &self,
        name: &str,
        arguments: Option<Map<String, Value>>,
    ) -> CallToolResult {
        // Generate a request ID for tracing
        let request_id = format!("{:x}", rand::random::<u64>());
        let ctx = self.create_context(&request_id);

        // Get arguments or empty object - convert Map to Value
        let args = arguments
            .map(Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));

        // Execute the tool
        let result = self.registry.execute(name, &ctx, args).await;

        match result {
            Ok(output) => self.to_mcp_result(output),
            Err(e) => {
                error!(error = %e, "Tool execution failed");
                CallToolResult {
                    content: vec![Content::text(format!("Error: {}", e))],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                }
            }
        }
    }
}

impl ServerHandler for GitLabMcpHandler {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: self.name.clone(),
                version: self.version.clone(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(
                "GitLab MCP Server - Access GitLab resources with fine-grained access control"
                    .to_string(),
            ),
        }
    }

    #[instrument(skip(self, _context))]
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        debug!("Listing tools");
        async move {
            Ok(ListToolsResult {
                tools: self.get_mcp_tools(),
                next_cursor: None,
            })
        }
    }

    #[instrument(skip(self, _context), fields(tool = %request.name))]
    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        debug!(?request.arguments, "Calling tool");
        async move { Ok(self.execute_tool(&request.name, request.arguments).await) }
    }
}
