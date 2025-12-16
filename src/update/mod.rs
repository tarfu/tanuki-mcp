//! Self-update functionality
//!
//! Provides automatic update checking and installation from GitHub releases.

mod checker;
mod config;
mod manager;

pub use checker::UpdateChecker;
pub use config::UpdateConfig;
pub use manager::UpdateManager;
