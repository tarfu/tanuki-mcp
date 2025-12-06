//! Stdio transport
//!
//! Runs the MCP server over standard input/output.

use crate::server::GitLabMcpHandler;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use tracing::info;

/// Run the MCP server using stdio transport
pub async fn run_stdio(handler: GitLabMcpHandler) -> anyhow::Result<()> {
    info!("Starting GitLab MCP server with stdio transport");

    // Create the stdio transport
    let transport = stdio();

    // Run the server
    let server = handler.serve(transport).await?;

    // Wait for completion
    server.waiting().await?;

    info!("GitLab MCP server stopped");
    Ok(())
}
