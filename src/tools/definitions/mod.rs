//! Tool definitions
//!
//! This module contains all GitLab MCP tool implementations.

pub mod branches;
pub mod commits;
pub mod groups;
pub mod issue_links;
pub mod issue_notes;
pub mod issues;
pub mod labels;
pub mod merge_requests;
pub mod milestones;
pub mod mr_discussions;
pub mod mr_drafts;
pub mod namespaces;
pub mod pipelines;
pub mod projects;
pub mod releases;
pub mod repository;
pub mod search;
pub mod tags;
pub mod users;
pub mod wiki;

use crate::tools::ToolRegistry;

/// Register all tools with the registry.
///
/// If the registry was constructed with [`ToolRegistry::new_filtered`], tools
/// denied by access control are skipped during registration.
pub fn register_all_tools(registry: &mut ToolRegistry) {
    registry.register_all_auto();
}
