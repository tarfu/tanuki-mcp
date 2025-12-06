//! Pattern matching for access control
//!
//! Provides regex-based pattern matching for allow/deny rules.

use crate::error::ConfigError;
use regex::Regex;

/// Compiled pattern matcher
#[derive(Debug)]
pub struct PatternMatcher {
    patterns: Vec<CompiledPattern>,
}

#[derive(Debug)]
struct CompiledPattern {
    source: String,
    regex: Regex,
}

impl PatternMatcher {
    /// Create a new pattern matcher from a list of regex patterns
    pub fn new(patterns: &[String]) -> Result<Self, ConfigError> {
        let mut compiled = Vec::with_capacity(patterns.len());

        for pattern in patterns {
            let regex = Regex::new(pattern).map_err(|e| ConfigError::InvalidPattern {
                pattern: pattern.clone(),
                reason: e.to_string(),
            })?;

            compiled.push(CompiledPattern {
                source: pattern.clone(),
                regex,
            });
        }

        Ok(Self { patterns: compiled })
    }

    /// Create an empty pattern matcher (matches nothing)
    pub fn empty() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Check if a tool name matches any pattern
    pub fn matches(&self, tool_name: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(tool_name))
    }

    /// Check if a tool name matches any pattern, returning the matching pattern
    pub fn find_match(&self, tool_name: &str) -> Option<&str> {
        self.patterns
            .iter()
            .find(|p| p.regex.is_match(tool_name))
            .map(|p| p.source.as_str())
    }

    /// Check if this matcher has any patterns
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Get the number of patterns
    pub fn len(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_matcher() {
        let matcher = PatternMatcher::empty();
        assert!(!matcher.matches("anything"));
        assert!(matcher.is_empty());
    }

    #[test]
    fn test_exact_match() {
        let matcher = PatternMatcher::new(&["^create_issue$".to_string()]).unwrap();
        assert!(matcher.matches("create_issue"));
        assert!(!matcher.matches("create_issue_note"));
        assert!(!matcher.matches("delete_issue"));
    }

    #[test]
    fn test_prefix_match() {
        let matcher = PatternMatcher::new(&["^delete_".to_string()]).unwrap();
        assert!(matcher.matches("delete_issue"));
        assert!(matcher.matches("delete_merge_request"));
        assert!(!matcher.matches("create_issue"));
    }

    #[test]
    fn test_suffix_match() {
        let matcher = PatternMatcher::new(&["_issue$".to_string()]).unwrap();
        assert!(matcher.matches("create_issue"));
        assert!(matcher.matches("delete_issue"));
        assert!(!matcher.matches("create_issue_note"));
    }

    #[test]
    fn test_wildcard_match() {
        let matcher = PatternMatcher::new(&[".*_note.*".to_string()]).unwrap();
        assert!(matcher.matches("create_issue_note"));
        assert!(matcher.matches("delete_mr_note"));
        assert!(!matcher.matches("create_issue"));
    }

    #[test]
    fn test_multiple_patterns() {
        let matcher =
            PatternMatcher::new(&["^delete_".to_string(), "^merge_".to_string()]).unwrap();

        assert!(matcher.matches("delete_issue"));
        assert!(matcher.matches("merge_merge_request"));
        assert!(!matcher.matches("create_issue"));
    }

    #[test]
    fn test_find_match() {
        let matcher =
            PatternMatcher::new(&["^delete_".to_string(), "^create_".to_string()]).unwrap();

        assert_eq!(matcher.find_match("delete_issue"), Some("^delete_"));
        assert_eq!(matcher.find_match("create_issue"), Some("^create_"));
        assert_eq!(matcher.find_match("list_issues"), None);
    }

    #[test]
    fn test_invalid_pattern() {
        let result = PatternMatcher::new(&["[invalid".to_string()]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidPattern { .. }
        ));
    }

    #[test]
    fn test_real_world_patterns() {
        // Common patterns users might configure
        let deny_destructive = PatternMatcher::new(&[
            "^delete_".to_string(),
            "^remove_".to_string(),
            "^force_".to_string(),
        ])
        .unwrap();

        assert!(deny_destructive.matches("delete_issue"));
        assert!(deny_destructive.matches("remove_label"));
        assert!(deny_destructive.matches("force_push"));
        assert!(!deny_destructive.matches("create_issue"));

        // Read-only pattern
        let allow_reads = PatternMatcher::new(&[
            "^list_".to_string(),
            "^get_".to_string(),
            "^search_".to_string(),
        ])
        .unwrap();

        assert!(allow_reads.matches("list_issues"));
        assert!(allow_reads.matches("get_merge_request"));
        assert!(allow_reads.matches("search_repositories"));
        assert!(!allow_reads.matches("create_issue"));
    }
}
