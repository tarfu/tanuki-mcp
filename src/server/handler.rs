//! MCP server handler
//!
//! Implements the MCP protocol handler for GitLab tools.

use crate::access_control::AccessResolver;
use crate::config::AppConfig;
use crate::dashboard::DashboardMetrics;
use crate::gitlab::GitLabClient;
use crate::tools::{ContentBlock, ToolContext, ToolOutput, ToolRegistry, definitions};
use base64::Engine;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, CompleteRequestParam, CompleteResult, CompletionInfo,
    Content, ErrorCode, GetPromptRequestParam, GetPromptResult, Implementation, InitializeResult,
    ListPromptsResult, ListResourcesResult, ListToolsResult, PaginatedRequestParam, Prompt,
    PromptArgument, PromptMessage, PromptMessageRole, PromptsCapability, ProtocolVersion,
    ReadResourceRequestParam, ReadResourceResult, ResourceContents, ResourcesCapability,
    ServerCapabilities, Tool, ToolsCapability,
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

    /// Get tool names for completion, filtered by prefix
    fn get_tool_completions(&self, prefix: &str) -> Vec<String> {
        self.registry
            .tools()
            .filter(|tool| tool.name.starts_with(prefix))
            .map(|tool| tool.name.to_string())
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

    /// Build the analyze_issue prompt
    async fn build_analyze_issue_prompt(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<GetPromptResult, McpError> {
        let args = arguments.ok_or_else(|| missing_argument("arguments required"))?;

        let project = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or_else(|| missing_argument("project"))?;

        let issue_iid = args
            .get("issue_iid")
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
            })
            .ok_or_else(|| missing_argument("issue_iid"))?;

        // Fetch issue details
        let encoded_project = GitLabClient::encode_project(project);
        let issue_endpoint = format!("/projects/{}/issues/{}", encoded_project, issue_iid);

        let issue: Value = self
            .gitlab
            .get(&issue_endpoint)
            .await
            .map_err(|e| internal_error(format!("Failed to fetch issue: {}", e)))?;

        // Fetch issue discussions
        let discussions_endpoint = format!("{}/discussions", issue_endpoint);
        let discussions: Value = self
            .gitlab
            .get(&discussions_endpoint)
            .await
            .unwrap_or_else(|_| serde_json::json!([]));

        // Build the prompt message
        let title = issue
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let description = issue
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let state = issue
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let author = issue
            .get("author")
            .and_then(|a| a.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let labels = issue
            .get("labels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let mut prompt_text = format!(
            "# Issue Analysis: {} #{}\n\n\
            **Project:** {}\n\
            **State:** {}\n\
            **Author:** {}\n\
            **Labels:** {}\n\n\
            ## Description\n\n{}\n\n",
            title, issue_iid, project, state, author, labels, description
        );

        // Add discussions if present
        if let Some(disc_array) = discussions.as_array() {
            if !disc_array.is_empty() {
                prompt_text.push_str("## Discussions\n\n");
                for (i, discussion) in disc_array.iter().enumerate() {
                    if let Some(notes) = discussion.get("notes").and_then(|n| n.as_array()) {
                        for note in notes {
                            let note_author = note
                                .get("author")
                                .and_then(|a| a.get("username"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let note_body = note.get("body").and_then(|v| v.as_str()).unwrap_or("");
                            prompt_text.push_str(&format!(
                                "### Comment {} by @{}\n\n{}\n\n",
                                i + 1,
                                note_author,
                                note_body
                            ));
                        }
                    }
                }
            }
        }

        prompt_text.push_str(
            "\n---\n\n\
            Please analyze this issue and provide:\n\
            1. A summary of the issue and its current status\n\
            2. Key points from the discussions\n\
            3. Suggested next steps or actions\n\
            4. Any potential blockers or concerns",
        );

        Ok(GetPromptResult {
            description: Some(format!("Analysis of issue #{} in {}", issue_iid, project)),
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                prompt_text,
            )],
        })
    }

    /// Build the review_merge_request prompt
    async fn build_review_mr_prompt(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<GetPromptResult, McpError> {
        let args = arguments.ok_or_else(|| missing_argument("arguments required"))?;

        let project = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or_else(|| missing_argument("project"))?;

        let mr_iid = args
            .get("mr_iid")
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
            })
            .ok_or_else(|| missing_argument("mr_iid"))?;

        // Fetch MR details
        let encoded_project = GitLabClient::encode_project(project);
        let mr_endpoint = format!("/projects/{}/merge_requests/{}", encoded_project, mr_iid);

        let mr: Value = self
            .gitlab
            .get(&mr_endpoint)
            .await
            .map_err(|e| internal_error(format!("Failed to fetch merge request: {}", e)))?;

        // Fetch MR changes (diff)
        let changes_endpoint = format!("{}/changes", mr_endpoint);
        let changes: Value = self
            .gitlab
            .get(&changes_endpoint)
            .await
            .unwrap_or_else(|_| serde_json::json!({"changes": []}));

        // Fetch MR discussions
        let discussions_endpoint = format!("{}/discussions", mr_endpoint);
        let discussions: Value = self
            .gitlab
            .get(&discussions_endpoint)
            .await
            .unwrap_or_else(|_| serde_json::json!([]));

        // Build the prompt message
        let title = mr
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let description = mr.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let state = mr
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let source_branch = mr
            .get("source_branch")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let target_branch = mr
            .get("target_branch")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let author = mr
            .get("author")
            .and_then(|a| a.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let labels = mr
            .get("labels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let mut prompt_text = format!(
            "# Merge Request Review: {} !{}\n\n\
            **Project:** {}\n\
            **State:** {}\n\
            **Author:** {}\n\
            **Source Branch:** {}\n\
            **Target Branch:** {}\n\
            **Labels:** {}\n\n\
            ## Description\n\n{}\n\n",
            title,
            mr_iid,
            project,
            state,
            author,
            source_branch,
            target_branch,
            labels,
            description
        );

        // Add changes summary
        if let Some(changes_array) = changes.get("changes").and_then(|c| c.as_array()) {
            prompt_text.push_str("## Changes\n\n");
            for change in changes_array {
                let old_path = change
                    .get("old_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let new_path = change
                    .get("new_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let diff = change.get("diff").and_then(|v| v.as_str()).unwrap_or("");

                if old_path != new_path && !old_path.is_empty() {
                    prompt_text.push_str(&format!("### {} → {}\n\n", old_path, new_path));
                } else {
                    prompt_text.push_str(&format!("### {}\n\n", new_path));
                }

                // Truncate very long diffs
                let truncated_diff = if diff.len() > 2000 {
                    format!("{}...\n(diff truncated)", &diff[..2000])
                } else {
                    diff.to_string()
                };
                prompt_text.push_str(&format!("```diff\n{}\n```\n\n", truncated_diff));
            }
        }

        // Add discussions if present
        if let Some(disc_array) = discussions.as_array() {
            let review_comments: Vec<_> = disc_array
                .iter()
                .filter(|d| {
                    d.get("notes")
                        .and_then(|n| n.as_array())
                        .map(|notes| {
                            notes
                                .iter()
                                .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("DiffNote"))
                        })
                        .unwrap_or(false)
                })
                .collect();

            if !review_comments.is_empty() {
                prompt_text.push_str("## Review Comments\n\n");
                for discussion in review_comments {
                    if let Some(notes) = discussion.get("notes").and_then(|n| n.as_array()) {
                        for note in notes {
                            let note_author = note
                                .get("author")
                                .and_then(|a| a.get("username"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let note_body = note.get("body").and_then(|v| v.as_str()).unwrap_or("");
                            let resolved = note
                                .get("resolved")
                                .and_then(|v| v.as_bool())
                                .map(|r| if r { " ✓" } else { "" })
                                .unwrap_or("");
                            prompt_text.push_str(&format!(
                                "- **@{}**{}: {}\n",
                                note_author, resolved, note_body
                            ));
                        }
                    }
                }
                prompt_text.push('\n');
            }
        }

        prompt_text.push_str(
            "\n---\n\n\
            Please review this merge request and provide:\n\
            1. A summary of the changes\n\
            2. Code quality assessment\n\
            3. Potential issues or concerns\n\
            4. Suggestions for improvement\n\
            5. Overall recommendation (approve/request changes)",
        );

        Ok(GetPromptResult {
            description: Some(format!(
                "Review of merge request !{} in {}",
                mr_iid, project
            )),
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                prompt_text,
            )],
        })
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
                completions: Some(Map::new()),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                prompts: Some(PromptsCapability {
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
                meta: None,
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

    #[instrument(skip(self, _context))]
    fn complete(
        &self,
        request: CompleteRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CompleteResult, McpError>> + Send + '_ {
        debug!(?request, "Processing completion request");
        async move {
            // Extract the argument name being completed
            let arg_name = &request.argument.name;
            let prefix = &request.argument.value;

            // Get completions based on what's being completed
            let values = match arg_name.as_str() {
                // For tool references, suggest tool names
                "name" => self.get_tool_completions(prefix),
                // For project parameters, we could suggest projects
                // but that would require an API call - return empty for now
                "project" | "project_id" => Vec::new(),
                // Default: no completions
                _ => Vec::new(),
            };

            let total = values.len() as u32;
            let has_more = values.len() > 100;
            let truncated = if has_more {
                values.into_iter().take(100).collect()
            } else {
                values
            };

            Ok(CompleteResult {
                completion: CompletionInfo {
                    values: truncated,
                    total: Some(total),
                    has_more: Some(has_more),
                },
            })
        }
    }

    /// List available resources
    ///
    /// Returns an empty list - resources are accessed directly by URI.
    /// The `gitlab://` URI scheme allows clients to read files from repositories.
    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
                meta: None,
            })
        }
    }

    /// Read a GitLab file resource by URI
    ///
    /// URI format: `gitlab://{project}/{path}?ref={branch}`
    /// - project: URL-encoded project path (e.g., `group%2Fsubgroup%2Fproject`)
    /// - path: File path within repository
    /// - ref: Optional git reference (branch, tag, commit) - defaults to HEAD
    #[instrument(skip(self, _context))]
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        debug!(uri = %request.uri, "Reading resource");
        async move {
            // Parse gitlab://project/path?ref=branch URI
            let (project, file_path, ref_name) = parse_gitlab_uri(&request.uri)?;

            // Build GitLab API endpoint
            let encoded_project = GitLabClient::encode_project(&project);
            let encoded_path = urlencoding::encode(&file_path);
            let ref_param = ref_name.as_deref().unwrap_or("HEAD");
            let endpoint = format!(
                "/projects/{}/repository/files/{}?ref={}",
                encoded_project,
                encoded_path,
                urlencoding::encode(ref_param)
            );

            // Fetch file content
            let result: serde_json::Value = self
                .gitlab
                .get(&endpoint)
                .await
                .map_err(|e| internal_error(format!("GitLab API error: {}", e)))?;

            // Decode base64 content if present
            let content = if let Some(content_str) = result.get("content").and_then(|c| c.as_str())
            {
                if result
                    .get("encoding")
                    .and_then(|e| e.as_str())
                    .map(|e| e == "base64")
                    .unwrap_or(false)
                {
                    // Decode base64
                    let decoded = base64::engine::general_purpose::STANDARD
                        .decode(content_str)
                        .map_err(|e| internal_error(format!("Failed to decode base64: {}", e)))?;
                    String::from_utf8(decoded)
                        .map_err(|e| internal_error(format!("Invalid UTF-8 content: {}", e)))?
                } else {
                    content_str.to_string()
                }
            } else {
                return Err(internal_error("No content in response"));
            };

            // Determine MIME type from file path
            let mime_type = guess_mime_type(&file_path);

            Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: request.uri,
                    mime_type: Some(mime_type),
                    text: content,
                    meta: None,
                }],
            })
        }
    }

    /// List available prompts
    ///
    /// Returns built-in workflow prompts for GitLab operations.
    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
        async move {
            Ok(ListPromptsResult {
                prompts: vec![
                    Prompt::new(
                        "analyze_issue",
                        Some("Analyze a GitLab issue with discussions and related MRs"),
                        Some(vec![
                            PromptArgument {
                                name: "project".to_string(),
                                title: Some("Project".to_string()),
                                description: Some(
                                    "Project path (e.g., 'group/project')".to_string(),
                                ),
                                required: Some(true),
                            },
                            PromptArgument {
                                name: "issue_iid".to_string(),
                                title: Some("Issue IID".to_string()),
                                description: Some("Issue internal ID number".to_string()),
                                required: Some(true),
                            },
                        ]),
                    ),
                    Prompt::new(
                        "review_merge_request",
                        Some("Review a merge request with changes and discussions"),
                        Some(vec![
                            PromptArgument {
                                name: "project".to_string(),
                                title: Some("Project".to_string()),
                                description: Some(
                                    "Project path (e.g., 'group/project')".to_string(),
                                ),
                                required: Some(true),
                            },
                            PromptArgument {
                                name: "mr_iid".to_string(),
                                title: Some("MR IID".to_string()),
                                description: Some("Merge request internal ID number".to_string()),
                                required: Some(true),
                            },
                        ]),
                    ),
                ],
                next_cursor: None,
                meta: None,
            })
        }
    }

    /// Get a specific prompt by name
    ///
    /// Builds workflow prompts that fetch relevant GitLab data.
    #[instrument(skip(self, _context))]
    fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, McpError>> + Send + '_ {
        debug!(name = %request.name, "Getting prompt");
        async move {
            match request.name.as_str() {
                "analyze_issue" => self.build_analyze_issue_prompt(request.arguments).await,
                "review_merge_request" => self.build_review_mr_prompt(request.arguments).await,
                _ => Err(method_not_found(&request.name)),
            }
        }
    }
}

/// Parse a gitlab:// URI into (project, path, ref_name)
///
/// URI format: `gitlab://{project}/{path}?ref={branch}`
/// - project: URL-encoded project path
/// - path: File path within repository
/// - ref: Optional git reference
fn parse_gitlab_uri(uri: &str) -> Result<(String, String, Option<String>), McpError> {
    // Check scheme
    let rest = uri
        .strip_prefix("gitlab://")
        .ok_or_else(|| invalid_resource_uri("URI must start with 'gitlab://'"))?;

    // Split query string
    let (path_part, query) = match rest.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (rest, None),
    };

    // Split into project and file path
    // The project can contain "/" so we need to be smart about this
    // Project is the first segment(s) until we find a valid file path
    // For simplicity, we require the project to be URL-encoded (group%2Fproject)
    // or just a single segment
    let parts: Vec<&str> = path_part.splitn(2, '/').collect();
    if parts.len() < 2 {
        return Err(invalid_resource_uri(
            "URI must contain project and file path",
        ));
    }

    let project = urlencoding::decode(parts[0])
        .map_err(|_| invalid_resource_uri("Invalid URL encoding in project"))?
        .to_string();
    let file_path = parts[1].to_string();

    // Parse ref from query string
    let ref_name = query.and_then(|q| {
        q.split('&').find_map(|param| {
            param
                .strip_prefix("ref=")
                .and_then(|v| urlencoding::decode(v).map(|s| s.to_string()).ok())
        })
    });

    Ok((project, file_path, ref_name))
}

/// Create an internal error McpError
fn internal_error(message: impl Into<Cow<'static, str>>) -> McpError {
    McpError {
        code: ErrorCode(-32603), // Internal error
        message: message.into(),
        data: None,
    }
}

/// Create an invalid resource URI error
fn invalid_resource_uri(message: impl Into<Cow<'static, str>>) -> McpError {
    McpError {
        code: ErrorCode(-32602), // Invalid params
        message: message.into(),
        data: None,
    }
}

/// Create a missing argument error
fn missing_argument(arg_name: &str) -> McpError {
    McpError {
        code: ErrorCode(-32602), // Invalid params
        message: format!("Missing required argument: {}", arg_name).into(),
        data: None,
    }
}

/// Create a method not found error
fn method_not_found(method_name: &str) -> McpError {
    McpError {
        code: ErrorCode(-32601), // Method not found
        message: format!("Unknown prompt: {}", method_name).into(),
        data: None,
    }
}

/// Guess MIME type from file path
fn guess_mime_type(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        // Text/code files
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "js" => "text/javascript",
        "ts" => "text/typescript",
        "jsx" => "text/javascript",
        "tsx" => "text/typescript",
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "toml" => "text/toml",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "xml" => "text/xml",
        "sql" => "text/x-sql",
        "sh" => "text/x-sh",
        "bash" => "text/x-sh",
        "zsh" => "text/x-sh",
        "c" => "text/x-c",
        "cpp" | "cc" | "cxx" => "text/x-c++",
        "h" => "text/x-c",
        "hpp" => "text/x-c++",
        "java" => "text/x-java",
        "go" => "text/x-go",
        "rb" => "text/x-ruby",
        "php" => "text/x-php",
        "swift" => "text/x-swift",
        "kt" | "kts" => "text/x-kotlin",
        "scala" => "text/x-scala",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "dockerfile" => "text/x-dockerfile",
        "makefile" => "text/x-makefile",
        // Default
        _ => "text/plain",
    }
    .to_string()
}
