//! GitLab API response types
//!
//! Common types used across multiple GitLab API endpoints.

use serde::{Deserialize, Serialize};

/// GitLab user (author, assignee, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
}

/// GitLab project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub path_with_namespace: String,
    #[serde(default)]
    pub description: Option<String>,
    pub visibility: String,
    pub web_url: String,
    #[serde(default)]
    pub ssh_url_to_repo: Option<String>,
    #[serde(default)]
    pub http_url_to_repo: Option<String>,
    #[serde(default)]
    pub default_branch: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub last_activity_at: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

/// GitLab namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub kind: String, // "user" or "group"
    pub full_path: String,
    #[serde(default)]
    pub parent_id: Option<u64>,
    #[serde(default)]
    pub web_url: Option<String>,
}

/// GitLab issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub state: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub closed_at: Option<String>,
    pub author: User,
    #[serde(default)]
    pub assignees: Vec<User>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    pub web_url: String,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub due_date: Option<String>,
}

/// GitLab merge request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub source_project_id: u64,
    #[serde(default)]
    pub target_project_id: Option<u64>,
    pub author: User,
    #[serde(default)]
    pub assignees: Vec<User>,
    #[serde(default)]
    pub reviewers: Vec<User>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub merged_at: Option<String>,
    #[serde(default)]
    pub closed_at: Option<String>,
    #[serde(default)]
    pub merged_by: Option<User>,
    pub web_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub merge_status: Option<String>,
    #[serde(default)]
    pub detailed_merge_status: Option<String>,
    #[serde(default)]
    pub sha: Option<String>,
    #[serde(default)]
    pub merge_commit_sha: Option<String>,
    #[serde(default)]
    pub squash_commit_sha: Option<String>,
}

/// GitLab milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: u64,
    pub iid: u64,
    #[serde(default)]
    pub project_id: Option<u64>,
    #[serde(default)]
    pub group_id: Option<u64>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub state: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
}

/// GitLab pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: u64,
    #[serde(default)]
    pub iid: Option<u64>,
    pub project_id: u64,
    pub status: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    #[serde(default)]
    pub source: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    pub web_url: String,
    #[serde(default)]
    pub user: Option<User>,
}

/// GitLab pipeline job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub stage: String,
    pub status: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    #[serde(default)]
    pub tag: bool,
    pub created_at: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    pub web_url: String,
    #[serde(default)]
    pub user: Option<User>,
    pub pipeline: PipelineRef,
}

/// Reference to a pipeline (minimal info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRef {
    pub id: u64,
    pub project_id: u64,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub status: String,
}

/// GitLab commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub title: String,
    #[serde(default)]
    pub message: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub authored_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_date: String,
    pub web_url: String,
    #[serde(default)]
    pub parent_ids: Vec<String>,
}

/// GitLab file/directory in repository tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String, // "tree" or "blob"
    pub path: String,
    pub mode: String,
}

/// GitLab file content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub file_name: String,
    pub file_path: String,
    pub size: u64,
    pub encoding: String,
    pub content: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub blob_id: String,
    pub commit_id: String,
    #[serde(default)]
    pub last_commit_id: Option<String>,
}

/// GitLab diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    pub old_path: String,
    pub new_path: String,
    #[serde(default)]
    pub a_mode: Option<String>,
    #[serde(default)]
    pub b_mode: Option<String>,
    pub diff: String,
    pub new_file: bool,
    pub renamed_file: bool,
    pub deleted_file: bool,
}

/// GitLab label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

/// GitLab note (comment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub body: String,
    pub author: User,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub resolvable: bool,
    #[serde(default)]
    pub resolved: bool,
    #[serde(default)]
    pub resolved_by: Option<User>,
}

/// GitLab discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    pub id: String,
    pub individual_note: bool,
    pub notes: Vec<Note>,
}

/// GitLab wiki page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    pub format: String,
    #[serde(default)]
    pub encoding: Option<String>,
}

/// GitLab release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub released_at: Option<String>,
    #[serde(default)]
    pub author: Option<User>,
    #[serde(default)]
    pub commit: Option<Commit>,
    #[serde(default)]
    pub assets: Option<ReleaseAssets>,
}

/// GitLab release assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAssets {
    pub count: u32,
    #[serde(default)]
    pub sources: Vec<ReleaseSource>,
    #[serde(default)]
    pub links: Vec<ReleaseLink>,
}

/// GitLab release source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSource {
    pub format: String,
    pub url: String,
}

/// GitLab release link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseLink {
    pub id: u64,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub link_type: Option<String>,
}

/// Pagination information from GitLab response headers
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total: Option<u32>,
    pub total_pages: Option<u32>,
    pub next_page: Option<u32>,
    pub prev_page: Option<u32>,
}
