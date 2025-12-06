//! Tools module
//!
//! Provides the framework for defining and executing GitLab MCP tools.

pub mod definitions;
pub mod executor;
pub mod registry;

pub use executor::{ContentBlock, ToolContext, ToolExecutor, ToolInfo, ToolOutput};
pub use registry::{RegisteredTool, ToolRegistry};

// Re-export the macro for convenience
pub use tanuki_mcp_macros::gitlab_tool;
