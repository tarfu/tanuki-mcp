//! Tool registry
//!
//! Manages the collection of available tools and their metadata.

use crate::access_control::{AccessControlled, AccessDecision, OperationType, ToolCategory};
use crate::error::{AccessDeniedError, ToolError};
use crate::tools::executor::ToolInfo;
use crate::tools::executor::{ToolContext, ToolExecutor, ToolOutput};
// async_trait required for dyn-compatibility with Box<dyn ToolHandler>
use async_trait::async_trait;
use schemars::Schema;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, instrument, warn};

/// Compile-time tool registration entry for auto-discovery
///
/// This struct is submitted via `inventory::submit!` by the `#[gitlab_tool]` macro,
/// allowing tools to be automatically registered without explicit calls.
pub struct ToolRegistration {
    /// Function that registers the tool with a registry
    pub register_fn: fn(&mut ToolRegistry),
}

inventory::collect!(ToolRegistration);

/// A registered tool with all its metadata
pub struct RegisteredTool {
    /// Tool name
    pub name: &'static str,
    /// Tool description
    pub description: &'static str,
    /// Tool category for access control
    pub category: ToolCategory,
    /// Operation type for access control
    pub operation: OperationType,
    /// JSON Schema for the tool's input
    pub input_schema: Schema,
    /// The tool handler
    handler: Box<dyn ToolHandler>,
}

/// Internal trait for type-erased tool handling
#[async_trait]
trait ToolHandler: Send + Sync {
    /// Execute the tool with raw JSON arguments
    async fn call(&self, ctx: &ToolContext, args: Value) -> Result<ToolOutput, ToolError>;

    /// Extract project from arguments (for access control)
    fn extract_project(&self, args: &Value) -> Option<String>;
}

/// Generic tool handler implementation
struct TypedToolHandler<T>
where
    T: ToolExecutor + DeserializeOwned + AccessControlled + 'static,
{
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedToolHandler<T>
where
    T: ToolExecutor + DeserializeOwned + AccessControlled + 'static,
{
    fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> ToolHandler for TypedToolHandler<T>
where
    T: ToolExecutor + DeserializeOwned + AccessControlled + Send + Sync + 'static,
{
    async fn call(&self, ctx: &ToolContext, args: Value) -> Result<ToolOutput, ToolError> {
        // Deserialize arguments into the tool struct
        let tool: T = serde_json::from_value(args).map_err(|e| {
            ToolError::InvalidArguments(format!("Failed to parse arguments: {}", e))
        })?;

        // Execute the tool
        tool.execute(ctx).await
    }

    fn extract_project(&self, args: &Value) -> Option<String> {
        // Try to deserialize and extract project
        // If deserialization fails, return None
        if let Ok(tool) = serde_json::from_value::<T>(args.clone()) {
            tool.extract_project()
        } else {
            None
        }
    }
}

/// Tool registry
pub struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
    by_category: HashMap<ToolCategory, Vec<String>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register<T>(&mut self)
    where
        T: ToolExecutor
            + DeserializeOwned
            + AccessControlled
            + schemars::JsonSchema
            + ToolInfo
            + Send
            + Sync
            + 'static,
    {
        let name = <T as ToolInfo>::name();
        let description = <T as ToolInfo>::description();
        let category = <T as ToolInfo>::category();
        let operation = <T as ToolInfo>::operation_type();

        // Generate JSON Schema
        let input_schema = schemars::schema_for!(T);

        let tool = RegisteredTool {
            name,
            description,
            category,
            operation,
            input_schema,
            handler: Box::new(TypedToolHandler::<T>::new()),
        };

        // Add to category index
        self.by_category
            .entry(category)
            .or_default()
            .push(name.to_string());

        // Add to main registry
        self.tools.insert(name.to_string(), tool);

        debug!(name = name, category = %category, "Registered tool");
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&RegisteredTool> {
        self.tools.get(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> impl Iterator<Item = &str> {
        self.tools.keys().map(|s| s.as_str())
    }

    /// Get all tools
    pub fn tools(&self) -> impl Iterator<Item = &RegisteredTool> {
        self.tools.values()
    }

    /// Get tools by category
    pub fn tools_in_category(&self, category: ToolCategory) -> Vec<&RegisteredTool> {
        self.by_category
            .get(&category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.tools.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Register all tools discovered via `#[gitlab_tool]` macro
    ///
    /// This method iterates over all `ToolRegistration` entries submitted at compile time
    /// and registers each tool with this registry.
    pub fn register_all_auto(&mut self) {
        for registration in inventory::iter::<ToolRegistration> {
            (registration.register_fn)(self);
        }
    }

    /// Execute a tool by name
    #[instrument(skip(self, ctx, args), fields(tool = %name))]
    pub async fn execute(
        &self,
        name: &str,
        ctx: &ToolContext,
        args: Value,
    ) -> Result<ToolOutput, ToolError> {
        let start = Instant::now();

        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        // Extract project for access control
        let project = tool.handler.extract_project(&args);

        // Check access control with enhanced error messages
        let decision = ctx
            .access
            .check(name, tool.category, tool.operation, project.as_deref());

        if let AccessDecision::Denied(reason) = decision {
            // Check if tool is globally denied vs project-specific denial
            let is_globally_denied =
                ctx.access
                    .is_globally_denied(name, tool.category, tool.operation);

            // Audit log: access denied (log before consuming reason)
            warn!(
                tool = %name,
                project = ?project,
                reason = %reason,
                request_id = %ctx.request_id,
                is_globally_denied = %is_globally_denied,
                "Access denied to tool"
            );

            // Convert reason to string for metrics before consuming
            let reason_str = reason.to_string();

            // Record to metrics with audit info if available
            if let Some(ref metrics) = ctx.metrics {
                let duration = start.elapsed();
                metrics.record_call_with_audit(
                    name,
                    tool.category,
                    project.as_deref(),
                    duration,
                    false,
                    Some(&ctx.request_id),
                    Some("denied"),
                    Some(&reason_str),
                );
            }

            let error = if is_globally_denied {
                // Tool is completely unavailable
                AccessDeniedError::globally_unavailable(name)
            } else if project.is_some()
                && ctx.access.has_project_specific_access(name, tool.category)
            {
                // Tool might be available for other projects
                AccessDeniedError::project_restricted_with_hint(name, project.as_deref().unwrap())
            } else {
                // Use the original reason
                AccessDeniedError::new(name, reason)
            };

            return Err(ToolError::AccessDenied(error));
        }

        // Execute the tool
        let result = tool.handler.call(ctx, args).await;

        // Record metrics with audit info if available
        if let Some(ref metrics) = ctx.metrics {
            let duration = start.elapsed();
            let success = result.is_ok() && !result.as_ref().map(|o| o.is_error).unwrap_or(false);
            let error_details = if !success {
                result.as_ref().err().map(|e| e.to_string())
            } else {
                None
            };
            metrics.record_call_with_audit(
                name,
                tool.category,
                project.as_deref(),
                duration,
                success,
                Some(&ctx.request_id),
                Some("allowed"),
                error_details.as_deref(),
            );
        }

        result
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests require actual tool implementations
    // These are basic structure tests

    #[test]
    fn test_empty_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_tool_not_found() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }
}
