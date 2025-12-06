//! Access control module
//!
//! Provides hierarchical, fine-grained access control for GitLab MCP tools.
//!
//! ## Access Control Model
//!
//! The access control system uses a hierarchical override model with the following
//! precedence (highest to lowest):
//!
//! 1. **Project-specific action override** - Explicit allow/deny for a specific action in a project
//! 2. **Global action override** - Explicit allow/deny for a specific action
//! 3. **Project-specific category** - Access level and patterns for a category in a project
//! 4. **Global category** - Access level and patterns for a category
//! 5. **Project-specific base** - Base access level for a project
//! 6. **Global base** - Base access level for all tools
//!
//! At each level, patterns are evaluated as follows:
//! - `allow` patterns are checked first and take precedence
//! - `deny` patterns are checked if no allow pattern matched
//! - If no pattern matched, the access level is used
//!
//! ## Example Configuration
//!
//! ```toml
//! [access_control]
//! all = "read"                    # Base: read-only
//! deny = ["delete_.*"]            # Global deny pattern
//!
//! [access_control.categories.issues]
//! level = "full"                  # Override: full access to issues
//!
//! [access_control.actions]
//! merge_merge_request = "deny"    # Block merging
//!
//! [access_control.projects."prod/app"]
//! all = "read"                    # Production is read-only
//! ```

pub mod patterns;
pub mod resolver;
pub mod types;

pub use patterns::PatternMatcher;
pub use resolver::{AccessDecision, AccessResolver};
pub use types::{AccessControlled, OperationType, ToolCategory};
