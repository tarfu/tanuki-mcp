//! Authentication provider trait
//!
//! Defines an extensible authentication system that currently supports
//! Personal Access Tokens and can be extended for OAuth2 in the future.

use crate::error::AuthError;
// async_trait required for dyn-compatibility with Box<dyn AuthProvider>
use async_trait::async_trait;

/// Authentication provider trait
///
/// Implementations provide authentication credentials for GitLab API requests.
/// This trait is designed to be extensible for future OAuth2 support.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the authentication header value
    ///
    /// Returns the value to be used in the `Authorization` header or
    /// `PRIVATE-TOKEN` header depending on the auth type.
    async fn get_auth_header(&self) -> Result<AuthHeader, AuthError>;

    /// Check if the credentials need to be refreshed
    ///
    /// For PAT, this always returns false.
    /// For OAuth2, this would check token expiration.
    fn needs_refresh(&self) -> bool;

    /// Refresh the credentials if needed
    ///
    /// For PAT, this is a no-op.
    /// For OAuth2, this would refresh the access token.
    async fn refresh(&mut self) -> Result<(), AuthError>;

    /// Get a description of the auth method (for logging)
    fn auth_type(&self) -> &'static str;
}

/// Authentication header to use with requests
#[derive(Debug, Clone)]
pub enum AuthHeader {
    /// Bearer token (used with OAuth2)
    Bearer(String),
    /// Private token (used with PAT)
    PrivateToken(String),
}

impl AuthHeader {
    /// Get the header name for this auth type
    pub fn header_name(&self) -> &'static str {
        match self {
            AuthHeader::Bearer(_) => "Authorization",
            AuthHeader::PrivateToken(_) => "PRIVATE-TOKEN",
        }
    }

    /// Get the header value for this auth type
    pub fn header_value(&self) -> String {
        match self {
            AuthHeader::Bearer(token) => format!("Bearer {}", token),
            AuthHeader::PrivateToken(token) => token.clone(),
        }
    }
}

/// Box type alias for auth providers
pub type BoxedAuthProvider = Box<dyn AuthProvider>;
