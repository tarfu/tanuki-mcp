//! Access control types
//!
//! Core types used by the access control system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Tool category for access control grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Issues,
    IssueLinks,
    IssueNotes,
    MergeRequests,
    MrDiscussions,
    MrDrafts,
    Repository,
    Branches,
    Commits,
    Projects,
    Namespaces,
    Labels,
    Wiki,
    Pipelines,
    Milestones,
    Releases,
    Users,
    Groups,
    Tags,
    Search,
}

impl ToolCategory {
    /// Get the category name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolCategory::Issues => "issues",
            ToolCategory::IssueLinks => "issue_links",
            ToolCategory::IssueNotes => "issue_notes",
            ToolCategory::MergeRequests => "merge_requests",
            ToolCategory::MrDiscussions => "mr_discussions",
            ToolCategory::MrDrafts => "mr_drafts",
            ToolCategory::Repository => "repository",
            ToolCategory::Branches => "branches",
            ToolCategory::Commits => "commits",
            ToolCategory::Projects => "projects",
            ToolCategory::Namespaces => "namespaces",
            ToolCategory::Labels => "labels",
            ToolCategory::Wiki => "wiki",
            ToolCategory::Pipelines => "pipelines",
            ToolCategory::Milestones => "milestones",
            ToolCategory::Releases => "releases",
            ToolCategory::Users => "users",
            ToolCategory::Groups => "groups",
            ToolCategory::Tags => "tags",
            ToolCategory::Search => "search",
        }
    }

    /// Try to parse a category from a string
    pub fn try_parse(s: &str) -> Option<Self> {
        match s {
            "issues" => Some(ToolCategory::Issues),
            "issue_links" => Some(ToolCategory::IssueLinks),
            "issue_notes" => Some(ToolCategory::IssueNotes),
            "merge_requests" => Some(ToolCategory::MergeRequests),
            "mr_discussions" => Some(ToolCategory::MrDiscussions),
            "mr_drafts" => Some(ToolCategory::MrDrafts),
            "repository" => Some(ToolCategory::Repository),
            "branches" => Some(ToolCategory::Branches),
            "commits" => Some(ToolCategory::Commits),
            "projects" => Some(ToolCategory::Projects),
            "namespaces" => Some(ToolCategory::Namespaces),
            "labels" => Some(ToolCategory::Labels),
            "wiki" => Some(ToolCategory::Wiki),
            "pipelines" => Some(ToolCategory::Pipelines),
            "milestones" => Some(ToolCategory::Milestones),
            "releases" => Some(ToolCategory::Releases),
            "users" => Some(ToolCategory::Users),
            "groups" => Some(ToolCategory::Groups),
            "tags" => Some(ToolCategory::Tags),
            "search" => Some(ToolCategory::Search),
            _ => None,
        }
    }

    /// Get all categories
    pub fn all() -> &'static [ToolCategory] {
        &[
            ToolCategory::Issues,
            ToolCategory::IssueLinks,
            ToolCategory::IssueNotes,
            ToolCategory::MergeRequests,
            ToolCategory::MrDiscussions,
            ToolCategory::MrDrafts,
            ToolCategory::Repository,
            ToolCategory::Branches,
            ToolCategory::Commits,
            ToolCategory::Projects,
            ToolCategory::Namespaces,
            ToolCategory::Labels,
            ToolCategory::Wiki,
            ToolCategory::Pipelines,
            ToolCategory::Milestones,
            ToolCategory::Releases,
            ToolCategory::Users,
            ToolCategory::Groups,
            ToolCategory::Tags,
            ToolCategory::Search,
        ]
    }
}

impl fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Operation type for determining read vs write access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Read operations (get, list, search)
    Read,
    /// Write operations (create, update)
    Write,
    /// Delete operations
    Delete,
    /// Execute operations (merge, retry, cancel, play)
    Execute,
}

impl OperationType {
    /// Check if this operation is read-only
    pub const fn is_read_only(&self) -> bool {
        matches!(self, OperationType::Read)
    }

    /// Check if this operation modifies data
    pub const fn is_mutating(&self) -> bool {
        !self.is_read_only()
    }

    /// Get the operation name as a string
    pub const fn as_str(&self) -> &'static str {
        match self {
            OperationType::Read => "read",
            OperationType::Write => "write",
            OperationType::Delete => "delete",
            OperationType::Execute => "execute",
        }
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trait for types that can be access controlled
pub trait AccessControlled {
    /// Get the tool name
    fn tool_name(&self) -> &'static str;

    /// Get the tool's category
    fn category(&self) -> ToolCategory;

    /// Get the operation type
    fn operation_type(&self) -> OperationType;

    /// Extract project identifier from the tool's arguments
    /// Returns None if the tool doesn't operate on a specific project
    fn extract_project(&self) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_roundtrip() {
        for category in ToolCategory::all() {
            let s = category.as_str();
            let parsed = ToolCategory::try_parse(s).unwrap();
            assert_eq!(*category, parsed);
        }
    }

    #[test]
    fn test_operation_type_read_only() {
        assert!(OperationType::Read.is_read_only());
        assert!(!OperationType::Write.is_read_only());
        assert!(!OperationType::Delete.is_read_only());
        assert!(!OperationType::Execute.is_read_only());
    }

    #[test]
    fn test_operation_type_mutating() {
        assert!(!OperationType::Read.is_mutating());
        assert!(OperationType::Write.is_mutating());
        assert!(OperationType::Delete.is_mutating());
        assert!(OperationType::Execute.is_mutating());
    }
}
