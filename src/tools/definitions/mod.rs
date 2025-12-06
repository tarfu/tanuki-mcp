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

/// Register all tools with the registry
pub fn register_all_tools(registry: &mut ToolRegistry) {
    issues::register(registry);
    issue_notes::register(registry);
    issue_links::register(registry);
    merge_requests::register(registry);
    mr_discussions::register(registry);
    mr_drafts::register(registry);
    repository::register(registry);
    branches::register(registry);
    commits::register(registry);
    pipelines::register(registry);
    projects::register(registry);
    namespaces::register(registry);
    labels::register(registry);
    milestones::register(registry);
    wiki::register(registry);
    releases::register(registry);
    users::register(registry);
    groups::register(registry);
    tags::register(registry);
    search::register(registry);
}
