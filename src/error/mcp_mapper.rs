//! MCP error code mapping.
//!
//! Maps application errors to MCP protocol errors with appropriate JSON-RPC error codes.
//!
//! # Strategy
//! - Protocol-level errors (tool not found, invalid params) → `Err(McpError)`
//! - Tool execution errors → `Ok(CallToolResult { is_error: true })`
//!
//! This distinction allows MCP clients to differentiate between:
//! - Problems with the request itself (protocol errors)
//! - Problems during tool execution (tool errors)

use rmcp::ErrorData as McpError;
use rmcp::model::ErrorCode;
use serde_json::{Value, json};
use std::borrow::Cow;

use super::{AccessDeniedError, GitLabError, ToolError};

/// Maps a `ToolError` to an MCP protocol error.
///
/// Use this for errors that should be returned as `Err(McpError)` rather than
/// `Ok(CallToolResult { is_error: true })`.
///
/// # When to use which
/// - **Protocol errors** (return `Err`): Tool not found, invalid arguments
/// - **Execution errors** (return `Ok` with `is_error: true`): GitLab API failures, access denied
///
/// # Example
/// ```ignore
/// match registry.execute(name, ctx, args).await {
///     Ok(output) => Ok(to_mcp_result(output)),
///     Err(ToolError::NotFound(name)) => Err(map_tool_error(&ToolError::NotFound(name))),
///     Err(e) => Ok(CallToolResult { is_error: Some(true), content: vec![...], ... })
/// }
/// ```
pub fn map_tool_error(error: &ToolError) -> McpError {
    match error {
        ToolError::NotFound(name) => McpError {
            code: ErrorCode::METHOD_NOT_FOUND,
            message: Cow::Owned(format!("Tool '{}' not found", name)),
            data: Some(json!({
                "tool": name,
                "error_type": "ToolNotFound"
            })),
        },

        ToolError::InvalidArguments(msg) => McpError {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::Owned(msg.clone()),
            data: Some(json!({
                "error_type": "InvalidArguments"
            })),
        },

        ToolError::MissingArgument(arg) => McpError {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::Owned(format!("Missing required argument: {}", arg)),
            data: Some(json!({
                "argument": arg,
                "error_type": "MissingArgument"
            })),
        },

        ToolError::Serialization(e) => McpError {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::Owned(format!("Invalid argument format: {}", e)),
            data: Some(json!({
                "error_type": "SerializationError"
            })),
        },

        ToolError::ExecutionFailed(msg) => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(msg.clone()),
            data: Some(json!({
                "error_type": "ExecutionFailed"
            })),
        },

        ToolError::GitLab(gitlab_err) => map_gitlab_error(gitlab_err),

        ToolError::AccessDenied(access_err) => map_access_denied_error(access_err),
    }
}

/// Maps a `GitLabError` to an MCP protocol error.
pub fn map_gitlab_error(error: &GitLabError) -> McpError {
    match error {
        GitLabError::Unauthorized => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Borrowed("GitLab authentication failed"),
            data: Some(json!({
                "error_type": "Unauthorized",
                "hint": "Check that your GitLab token is valid and not expired"
            })),
        },

        GitLabError::Forbidden { action } => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!(
                "Forbidden: insufficient permissions for {}",
                action
            )),
            data: Some(json!({
                "error_type": "Forbidden",
                "action": action
            })),
        },

        GitLabError::NotFound { resource } => McpError {
            code: ErrorCode::RESOURCE_NOT_FOUND,
            message: Cow::Owned(format!("Resource not found: {}", resource)),
            data: Some(json!({
                "error_type": "NotFound",
                "resource": resource
            })),
        },

        GitLabError::RateLimited { retry_after } => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("Rate limited, retry after {} seconds", retry_after)),
            data: Some(json!({
                "error_type": "RateLimited",
                "retry_after": retry_after
            })),
        },

        GitLabError::Timeout { timeout_secs } => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("Request timeout after {} seconds", timeout_secs)),
            data: Some(json!({
                "error_type": "Timeout",
                "timeout_secs": timeout_secs
            })),
        },

        GitLabError::Api { status, message } => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("GitLab API error (HTTP {}): {}", status, message)),
            data: Some(json!({
                "error_type": "ApiError",
                "status": status
            })),
        },

        GitLabError::Request(e) => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("HTTP request failed: {}", e)),
            data: Some(json!({
                "error_type": "RequestError"
            })),
        },

        GitLabError::InvalidResponse(msg) => McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("Invalid response from GitLab: {}", msg)),
            data: Some(json!({
                "error_type": "InvalidResponse"
            })),
        },
    }
}

/// Maps an `AccessDeniedError` to an MCP protocol error.
pub fn map_access_denied_error(error: &AccessDeniedError) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::Owned(error.to_string()),
        data: Some(json!({
            "error_type": "AccessDenied",
            "tool": error.tool,
            "reason": error.reason
        })),
    }
}

/// Creates an MCP error for an internal server error.
pub fn internal_error(message: impl Into<String>) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::Owned(message.into()),
        data: Some(json!({
            "error_type": "InternalError"
        })),
    }
}

/// Creates an MCP error for invalid parameters.
pub fn invalid_params(message: impl Into<String>) -> McpError {
    McpError {
        code: ErrorCode::INVALID_PARAMS,
        message: Cow::Owned(message.into()),
        data: None,
    }
}

/// Creates an MCP error for method not found.
pub fn method_not_found(method: impl Into<String>) -> McpError {
    let method = method.into();
    McpError {
        code: ErrorCode::METHOD_NOT_FOUND,
        message: Cow::Owned(format!("Method '{}' not found", method)),
        data: Some(json!({
            "method": method
        })),
    }
}

/// Converts error data to a JSON value for inclusion in error responses.
pub fn error_to_json(error: &ToolError) -> Value {
    let mcp_error = map_tool_error(error);
    json!({
        "code": mcp_error.code.0,
        "message": mcp_error.message,
        "data": mcp_error.data
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_tool_not_found() {
        let error = ToolError::NotFound("unknown_tool".into());
        let mcp_error = map_tool_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::METHOD_NOT_FOUND);
        assert!(mcp_error.message.contains("unknown_tool"));
        assert!(mcp_error.data.is_some());
    }

    #[test]
    fn test_map_invalid_arguments() {
        let error = ToolError::InvalidArguments("project must be a string".into());
        let mcp_error = map_tool_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::INVALID_PARAMS);
        assert!(mcp_error.message.contains("project"));
    }

    #[test]
    fn test_map_missing_argument() {
        let error = ToolError::MissingArgument("project".into());
        let mcp_error = map_tool_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::INVALID_PARAMS);
        assert!(mcp_error.message.contains("project"));
    }

    #[test]
    fn test_map_gitlab_unauthorized() {
        let error = GitLabError::Unauthorized;
        let mcp_error = map_gitlab_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::INTERNAL_ERROR);
        assert!(mcp_error.message.contains("authentication"));
    }

    #[test]
    fn test_map_gitlab_rate_limited() {
        let error = GitLabError::RateLimited { retry_after: 60 };
        let mcp_error = map_gitlab_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::INTERNAL_ERROR);
        assert!(mcp_error.message.contains("60"));

        let data = mcp_error.data.unwrap();
        assert_eq!(data["retry_after"], 60);
    }

    #[test]
    fn test_map_gitlab_not_found() {
        let error = GitLabError::NotFound {
            resource: "project".into(),
        };
        let mcp_error = map_gitlab_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::RESOURCE_NOT_FOUND);
        assert!(mcp_error.message.contains("project"));
    }

    #[test]
    fn test_map_access_denied() {
        let error = AccessDeniedError::new("delete_issue", "write operations disabled");
        let mcp_error = map_access_denied_error(&error);

        assert_eq!(mcp_error.code, ErrorCode::INTERNAL_ERROR);
        assert!(mcp_error.message.contains("delete_issue"));

        let data = mcp_error.data.unwrap();
        assert_eq!(data["tool"], "delete_issue");
    }

    #[test]
    fn test_helper_functions() {
        let err = internal_error("Something went wrong");
        assert_eq!(err.code, ErrorCode::INTERNAL_ERROR);

        let err = invalid_params("Missing field");
        assert_eq!(err.code, ErrorCode::INVALID_PARAMS);

        let err = method_not_found("unknown_method");
        assert_eq!(err.code, ErrorCode::METHOD_NOT_FOUND);
    }
}
