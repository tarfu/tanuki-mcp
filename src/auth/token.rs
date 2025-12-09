//! Personal Access Token authentication
//!
//! Simple authentication using GitLab Personal Access Tokens.

use crate::auth::provider::{AuthHeader, AuthProvider};
use crate::error::AuthError;
use async_trait::async_trait;
use std::sync::Arc;

/// Personal Access Token authentication provider
#[derive(Debug, Clone)]
pub struct PatProvider {
    token: Arc<str>,
}

impl PatProvider {
    /// Create a new PAT provider
    pub fn new(token: impl Into<String>) -> Result<Self, AuthError> {
        let token = token.into();

        // Basic validation
        if token.is_empty() {
            return Err(AuthError::InvalidToken);
        }

        // GitLab PATs have specific prefixes (glpat- for newer tokens)
        // but we don't enforce this as older tokens may not have it
        Ok(Self {
            token: token.into(),
        })
    }

    /// Create from environment variable
    ///
    /// Checks GITLAB_TOKEN, GITLAB_PRIVATE_TOKEN, and GITLAB_ACCESS_TOKEN
    /// in order of precedence.
    pub fn from_env() -> Result<Self, AuthError> {
        for var in &[
            "GITLAB_TOKEN",
            "GITLAB_PRIVATE_TOKEN",
            "GITLAB_ACCESS_TOKEN",
        ] {
            if let Ok(token) = std::env::var(var)
                && !token.is_empty()
            {
                return Self::new(token);
            }
        }

        Err(AuthError::NotConfigured)
    }
}

#[async_trait]
impl AuthProvider for PatProvider {
    async fn get_auth_header(&self) -> Result<AuthHeader, AuthError> {
        Ok(AuthHeader::PrivateToken(Arc::clone(&self.token)))
    }

    fn needs_refresh(&self) -> bool {
        // PATs don't expire automatically (unless revoked)
        false
    }

    async fn refresh(&mut self) -> Result<(), AuthError> {
        // PATs can't be refreshed
        Ok(())
    }

    fn auth_type(&self) -> &'static str {
        "Personal Access Token"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pat_provider_new() {
        let provider = PatProvider::new("glpat-xxxx").unwrap();
        assert_eq!(&*provider.token, "glpat-xxxx");
    }

    #[test]
    fn test_pat_provider_empty_token() {
        let result = PatProvider::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    #[tokio::test]
    async fn test_pat_provider_auth_header() {
        let provider = PatProvider::new("test-token").unwrap();
        let header = provider.get_auth_header().await.unwrap();

        assert!(matches!(header, AuthHeader::PrivateToken(_)));
        assert_eq!(header.header_name(), "PRIVATE-TOKEN");
        assert_eq!(header.header_value(), "test-token");
    }

    #[test]
    fn test_pat_provider_no_refresh_needed() {
        let provider = PatProvider::new("test-token").unwrap();
        assert!(!provider.needs_refresh());
    }
}
