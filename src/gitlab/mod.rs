//! GitLab API module
//!
//! Provides a typed client for interacting with the GitLab REST API.

pub mod client;
pub mod types;

pub use client::GitLabClient;
pub use types::*;
