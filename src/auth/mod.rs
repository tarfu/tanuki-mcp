//! Authentication module
//!
//! Provides authentication mechanisms for GitLab API access.
//! Currently supports Personal Access Tokens, with the architecture
//! designed to easily support OAuth2 in the future.

pub mod provider;
pub mod token;

pub use provider::{AuthHeader, AuthProvider, BoxedAuthProvider};
pub use token::PatProvider;

use crate::config::GitLabConfig;
use crate::error::AuthError;

/// Create an auth provider from configuration
pub fn create_auth_provider(config: &GitLabConfig) -> Result<BoxedAuthProvider, AuthError> {
    if let Some(token) = &config.token {
        Ok(Box::new(PatProvider::new(token.clone())?))
    } else {
        // Try environment variables
        Ok(Box::new(PatProvider::from_env()?))
    }
}
