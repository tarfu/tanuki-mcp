//! Error types for tanuki-mcp
//!
//! This module defines the error hierarchy used throughout the application.
//! We use `thiserror` for library-style errors that are part of the API,
//! and convert to appropriate MCP error responses at the boundary.

use thiserror::Error;

/// Top-level application error
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("GitLab API error: {0}")]
    GitLab(#[from] GitLabError),

    #[error("Access denied: {0}")]
    AccessDenied(#[from] AccessDeniedError),

    #[error("Tool execution error: {0}")]
    Tool(#[from] ToolError),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(String),

    #[error("Invalid configuration: {message}")]
    Invalid { message: String },

    #[error("Missing required configuration: {field}")]
    Missing { field: String },

    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidPattern { pattern: String, reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// GitLab API specific errors
#[derive(Error, Debug)]
pub enum GitLabError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("GitLab API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Unauthorized: invalid or expired token")]
    Unauthorized,

    #[error("Forbidden: insufficient permissions for {action}")]
    Forbidden { action: String },

    #[error("Invalid response from GitLab: {0}")]
    InvalidResponse(String),

    #[error("Request timeout after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
}

impl GitLabError {
    /// Create an appropriate error from an HTTP status code and response body
    pub fn from_response(status: u16, body: &str) -> Self {
        match status {
            401 => GitLabError::Unauthorized,
            403 => GitLabError::Forbidden {
                action: "this operation".into(),
            },
            404 => GitLabError::NotFound {
                resource: "requested resource".into(),
            },
            429 => {
                // Try to parse retry-after from body, default to 60
                GitLabError::RateLimited { retry_after: 60 }
            }
            _ => GitLabError::Api {
                status,
                message: if body.is_empty() {
                    format!("HTTP {}", status)
                } else {
                    body.to_string()
                },
            },
        }
    }
}

/// Access control errors
#[derive(Error, Debug)]
#[error("Access denied for tool '{tool}': {reason}")]
pub struct AccessDeniedError {
    pub tool: String,
    pub reason: String,
}

impl AccessDeniedError {
    pub fn new(tool: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: reason.into(),
        }
    }

    pub fn read_only(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: "write operations are not permitted in read-only mode".into(),
        }
    }

    pub fn denied_by_pattern(tool: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: format!("denied by pattern '{}'", pattern.into()),
        }
    }

    pub fn category_disabled(tool: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: format!("category '{}' is disabled", category.into()),
        }
    }

    pub fn project_restricted(tool: impl Into<String>, project: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: format!("not permitted for project '{}'", project.into()),
        }
    }

    /// Create an error indicating the tool is denied for this project but may be available for others
    pub fn project_restricted_with_hint(
        tool: impl Into<String>,
        project: impl Into<String>,
    ) -> Self {
        Self {
            tool: tool.into(),
            reason: format!(
                "not allowed for project '{}', but may be available for other projects",
                project.into()
            ),
        }
    }

    /// Create an error indicating the tool is completely unavailable
    pub fn globally_unavailable(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            reason: "this tool is not available".into(),
        }
    }
}

/// Tool execution errors
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("GitLab API error: {0}")]
    GitLab(#[from] GitLabError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(#[from] AccessDeniedError),
}

/// Transport layer errors
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("HTTP server error: {0}")]
    Http(String),
}

/// Authentication errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("No authentication configured")]
    NotConfigured,

    #[error("Invalid token format")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Authentication failed: {0}")]
    Failed(String),
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;

/// Result type alias for tool operations
pub type ToolResult<T> = std::result::Result<T, ToolError>;

/// Result type alias for GitLab API operations
pub type GitLabResult<T> = std::result::Result<T, GitLabError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_error_from_response() {
        assert!(matches!(
            GitLabError::from_response(401, ""),
            GitLabError::Unauthorized
        ));

        assert!(matches!(
            GitLabError::from_response(403, ""),
            GitLabError::Forbidden { .. }
        ));

        assert!(matches!(
            GitLabError::from_response(404, ""),
            GitLabError::NotFound { .. }
        ));

        assert!(matches!(
            GitLabError::from_response(429, ""),
            GitLabError::RateLimited { .. }
        ));

        let api_err = GitLabError::from_response(500, "Internal server error");
        assert!(matches!(api_err, GitLabError::Api { status: 500, .. }));
    }

    #[test]
    fn test_access_denied_constructors() {
        let err = AccessDeniedError::read_only("create_issue");
        assert!(err.reason.contains("read-only"));

        let err = AccessDeniedError::denied_by_pattern("delete_issue", "delete_.*");
        assert!(err.reason.contains("delete_.*"));

        let err = AccessDeniedError::category_disabled("list_wiki", "wiki");
        assert!(err.reason.contains("wiki"));

        let err = AccessDeniedError::project_restricted("merge_mr", "prod/app");
        assert!(err.reason.contains("prod/app"));
    }
}
