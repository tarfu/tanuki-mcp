//! Secret string type for safe token handling.
//!
//! Provides a wrapper type that prevents accidental logging of sensitive values.

use serde::Deserialize;
use std::fmt;

/// A wrapper for secrets that prevents accidental logging.
///
/// `SecretString` ensures that sensitive values like API tokens and passwords
/// are not accidentally exposed through debug output, logs, or error messages.
///
/// # Features
/// - `Debug` and `Display` implementations show `[REDACTED]` instead of the value
/// - Explicit `expose_secret()` method required to access the actual value
/// - Clears memory on drop (best-effort, not cryptographically secure)
///
/// # Example
/// ```ignore
/// let token = SecretString::new("my-secret-token");
///
/// // Debug output shows [REDACTED]
/// println!("{:?}", token);  // Output: [REDACTED]
///
/// // Explicit access required
/// let value = token.expose_secret();
/// ```
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    /// Create a new secret from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Explicitly expose the secret value.
    ///
    /// Use this method only when the secret value is actually needed,
    /// such as when constructing authentication headers.
    #[inline]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        // Best-effort memory clearing
        // Note: This is not cryptographically secure as the compiler may optimize this away
        // or the value may have been copied elsewhere in memory.
        // For production systems requiring secure memory handling, consider using
        // the `zeroize` crate or platform-specific secure memory APIs.
        self.0.clear();
        self.0.shrink_to_fit();
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(SecretString::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_redacted() {
        let secret = SecretString::new("my-secret-token");
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "[REDACTED]");
        assert!(!debug_output.contains("my-secret-token"));
    }

    #[test]
    fn test_display_redacted() {
        let secret = SecretString::new("my-secret-token");
        let display_output = format!("{}", secret);
        assert_eq!(display_output, "[REDACTED]");
    }

    #[test]
    fn test_expose_secret() {
        let secret = SecretString::new("my-secret-token");
        assert_eq!(secret.expose_secret(), "my-secret-token");
    }

    #[test]
    fn test_clone() {
        let secret = SecretString::new("my-secret-token");
        let cloned = secret.clone();
        assert_eq!(cloned.expose_secret(), "my-secret-token");
    }

    #[test]
    fn test_deserialize() {
        let json = r#""test-token""#;
        let secret: SecretString = serde_json::from_str(json).unwrap();
        assert_eq!(secret.expose_secret(), "test-token");
    }
}
