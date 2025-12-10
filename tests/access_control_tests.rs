//! Comprehensive access control integration tests
//!
//! This test suite covers all combinations of:
//! - Access levels (None, Read, Full)
//! - Operation types (Read, Write, Delete, Execute)
//! - Tool categories (21 categories)
//! - Pattern matching (allow/deny regex)
//! - Hierarchical resolution (6 levels)
//! - Project-specific overrides
//!
//! IMPORTANT: The access control system has the following behavior:
//! - Default AccessLevel is None (not Read)
//! - Category level=None means "no decision at this level" (falls through to base)
//! - To actually block at category level, use deny patterns
//! - Allow patterns override deny patterns at the same level
//! - Higher levels in hierarchy override lower levels

use std::collections::HashMap;
use tanuki_mcp::access_control::{AccessResolver, OperationType, ToolCategory};
use tanuki_mcp::config::{
    AccessControlConfig, AccessLevel, ActionPermission, CategoryAccessConfig, ProjectAccessConfig,
};

// =============================================================================
// Test Helpers
// =============================================================================

fn config_with_level(level: AccessLevel) -> AccessControlConfig {
    AccessControlConfig {
        all: level,
        deny: vec![],
        allow: vec![],
        categories: HashMap::new(),
        actions: HashMap::new(),
        projects: HashMap::new(),
    }
}

// =============================================================================
// 1. Access Level × Operation Type Matrix Tests (12 tests)
// =============================================================================

mod access_level_operation_matrix {
    use super::*;

    // --- AccessLevel::None tests ---

    #[test]
    fn test_none_denies_read() {
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_none_denies_write() {
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_none_denies_delete() {
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_none_denies_execute() {
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "merge_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Execute,
                    None
                )
                .is_denied()
        );
    }

    // --- AccessLevel::Read tests ---

    #[test]
    fn test_read_allows_read() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_read_denies_write() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_read_denies_delete() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_read_denies_execute() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "merge_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Execute,
                    None
                )
                .is_denied()
        );
    }

    // --- AccessLevel::Deny tests ---

    #[test]
    fn test_deny_denies_read() {
        let config = config_with_level(AccessLevel::Deny);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_deny_denies_write() {
        let config = config_with_level(AccessLevel::Deny);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_deny_denies_delete() {
        let config = config_with_level(AccessLevel::Deny);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_deny_denies_execute() {
        let config = config_with_level(AccessLevel::Deny);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "merge_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Execute,
                    None
                )
                .is_denied()
        );
    }

    // --- AccessLevel::Full tests ---

    #[test]
    fn test_full_allows_read() {
        let config = config_with_level(AccessLevel::Full);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_full_allows_write() {
        let config = config_with_level(AccessLevel::Full);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_full_allows_delete() {
        let config = config_with_level(AccessLevel::Full);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_full_allows_execute() {
        let config = config_with_level(AccessLevel::Full);
        let resolver = AccessResolver::new(&config).unwrap();
        assert!(
            resolver
                .check(
                    "merge_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Execute,
                    None
                )
                .is_allowed()
        );
    }
}

// =============================================================================
// 2. Pattern Matching Tests (20 tests)
// =============================================================================

mod pattern_tests {
    use super::*;

    mod positive {
        use super::*;

        #[test]
        fn test_exact_match_allows() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["create_issue".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_prefix_match_allows() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^create_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_project",
                        ToolCategory::Projects,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_suffix_match_allows() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["_issue$".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_wildcard_allows() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec![".*draft.*".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "create_draft_note",
                        ToolCategory::MrDrafts,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_multiple_allow_patterns() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^list_".to_string(), "^get_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check("get_issue", ToolCategory::Issues, OperationType::Read, None)
                    .is_allowed()
            );
        }

        #[test]
        fn test_allow_overrides_deny_at_same_level() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_.*".to_string()];
            config.allow = vec!["delete_issue".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // delete_issue should be allowed (allow overrides deny)
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            // Other deletes should still be denied
            assert!(
                resolver
                    .check(
                        "delete_project",
                        ToolCategory::Projects,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_pattern_case_sensitive() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["Create_Issue".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // Should NOT match because case differs
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_pattern_matches_substring_by_default() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["issue".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // "issue" matches as substring in "list_issues"
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_anchored_pattern_requires_exact_position() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^list_issues$".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "list_issues_by_label",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_alternation_pattern() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^(list|get)_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check("get_issue", ToolCategory::Issues, OperationType::Read, None)
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }
    }

    mod negative {
        use super::*;

        #[test]
        fn test_pattern_no_match_denied() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^list_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_deny_pattern_blocks() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_.*".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_multiple_deny_patterns() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["^delete_".to_string(), "^merge_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "merge_merge_request",
                        ToolCategory::MergeRequests,
                        OperationType::Execute,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_empty_patterns_use_level() {
            let config = config_with_level(AccessLevel::Read);
            let resolver = AccessResolver::new(&config).unwrap();
            // No patterns, so uses access level
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_deny_without_allow_fallback() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_issue".to_string()];
            // No allow patterns
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            // But other operations still allowed
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_pattern_match_everything() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec![".*".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_pattern_match_nothing() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["^$".to_string()]; // Only matches empty string
            let resolver = AccessResolver::new(&config).unwrap();
            // All tools should still work since pattern matches nothing
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_overlapping_patterns_first_deny_wins() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_issue".to_string(), "delete_.*".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // Both patterns would match, but result is same - denied
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_special_regex_chars_need_escaping() {
            let mut config = config_with_level(AccessLevel::None);
            // Without escaping, . matches any char
            config.allow = vec!["list.issues".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // "list.issues" pattern with unescaped . matches "list_issues"
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_deny_all_allow_specific() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec![".*".to_string()];
            config.allow = vec!["^list_issues$".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();
            // Only list_issues should be allowed
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check("get_issue", ToolCategory::Issues, OperationType::Read, None)
                    .is_denied()
            );
        }
    }
}

// =============================================================================
// 3. Hierarchical Resolution Tests (36 tests)
// =============================================================================

mod hierarchy_tests {
    use super::*;

    mod global_base {
        use super::*;

        #[test]
        fn test_base_none_denies_all_operations() {
            let config = config_with_level(AccessLevel::None);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "merge_merge_request",
                        ToolCategory::MergeRequests,
                        OperationType::Execute,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_base_read_allows_only_reads() {
            let config = config_with_level(AccessLevel::Read);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_base_full_allows_all() {
            let config = config_with_level(AccessLevel::Full);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
        }
    }

    mod global_patterns {
        use super::*;

        #[test]
        fn test_deny_pattern_overrides_full_base() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["^delete_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_allow_pattern_grants_access_on_none_base() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^list_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_allow_overrides_deny_same_level() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_.*".to_string()];
            config.allow = vec!["delete_issue_note".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue_note",
                        ToolCategory::IssueNotes,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_multiple_deny_patterns_all_apply() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["^delete_".to_string(), "^merge_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "merge_merge_request",
                        ToolCategory::MergeRequests,
                        OperationType::Execute,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_multiple_allow_patterns_any_grants() {
            let mut config = config_with_level(AccessLevel::None);
            config.allow = vec!["^list_".to_string(), "^get_".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check("get_issue", ToolCategory::Issues, OperationType::Read, None)
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_complex_pattern_interaction() {
            let mut config = config_with_level(AccessLevel::Read);
            config.deny = vec!["list_merge_requests".to_string()];
            config.allow = vec!["list_.*".to_string()];
            let resolver = AccessResolver::new(&config).unwrap();

            // Allow pattern matches list_merge_requests, so it should be allowed
            assert!(
                resolver
                    .check(
                        "list_merge_requests",
                        ToolCategory::MergeRequests,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }
    }

    mod category_override {
        use super::*;

        #[test]
        fn test_category_more_permissive_than_base() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            // Issues should have full access
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            // Other categories should still be read-only
            assert!(
                resolver
                    .check(
                        "delete_project",
                        ToolCategory::Projects,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_category_more_restrictive_than_base() {
            let mut config = config_with_level(AccessLevel::Full);
            config.categories.insert(
                "pipelines".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Read,
                    deny: vec![],
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            // Pipelines should be read-only
            assert!(
                resolver
                    .check(
                        "list_pipelines",
                        ToolCategory::Pipelines,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
            // Other categories should still be full
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_category_pattern_overrides_category_level() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "merge_requests".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec!["merge_merge_request".to_string()],
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            // MR operations should be full except merge
            assert!(
                resolver
                    .check(
                        "create_merge_request",
                        ToolCategory::MergeRequests,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "merge_merge_request",
                        ToolCategory::MergeRequests,
                        OperationType::Execute,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_category_allow_overrides_category_deny() {
            let mut config = config_with_level(AccessLevel::Full);
            config.categories.insert(
                "pipelines".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Read,
                    deny: vec![".*".to_string()],
                    allow: vec!["retry_pipeline_job".to_string()],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "retry_pipeline_job",
                        ToolCategory::Pipelines,
                        OperationType::Execute,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_multiple_categories_independent() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            // Category with None level falls through to base
            config.categories.insert(
                "merge_requests".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![".*".to_string()], // Use deny pattern to actually block
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "list_merge_requests",
                        ToolCategory::MergeRequests,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "list_projects",
                        ToolCategory::Projects,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        // --- AccessLevel::Deny tests for categories ---

        #[test]
        fn test_category_deny_blocks_all_without_pattern_workaround() {
            // This is the main use case: explicit deny without needing deny = [".*"]
            let mut config = config_with_level(AccessLevel::Full);
            config.categories.insert(
                "wiki".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Deny, // Explicit deny - no pattern needed!
                    deny: vec![],
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_wiki_page",
                        ToolCategory::Wiki,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
            // Other categories still work
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_deny_vs_none_semantics() {
            // Demonstrates the difference: None falls through, Deny blocks
            let mut config = config_with_level(AccessLevel::Full);

            // Category with None - falls through to global Full
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![],
                    allow: vec![],
                },
            );

            // Category with Deny - explicitly blocks
            config.categories.insert(
                "wiki".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Deny,
                    deny: vec![],
                    allow: vec![],
                },
            );

            let resolver = AccessResolver::new(&config).unwrap();

            // Issues with None level falls through to Full base - allowed!
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );

            // Wiki with Deny level explicitly blocks
            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_deny_with_allow_pattern_override() {
            // Allow pattern should still override Deny level
            let mut config = config_with_level(AccessLevel::Full);
            config.categories.insert(
                "wiki".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Deny,
                    deny: vec![],
                    allow: vec!["list_wiki_pages".to_string()], // This specific tool allowed
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            // Allowed by pattern despite Deny level
            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );

            // Still denied (no pattern match)
            assert!(
                resolver
                    .check(
                        "create_wiki_page",
                        ToolCategory::Wiki,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_project_category_deny() {
            // Deny at project category level
            let mut config = config_with_level(AccessLevel::Full);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.categories.insert(
                "wiki".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Deny,
                    deny: vec![],
                    allow: vec![],
                },
            );
            config
                .projects
                .insert("docs/internal".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Wiki denied for this project
            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        Some("docs/internal")
                    )
                    .is_denied()
            );

            // Wiki allowed globally (no project context)
            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_category_deny_pattern_blocks() {
            // To block at category level, use deny patterns (level=None just falls through)
            let mut config = config_with_level(AccessLevel::Full);
            config.categories.insert(
                "wiki".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![".*".to_string()], // This actually blocks
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_wiki_pages",
                        ToolCategory::Wiki,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_wiki_page",
                        ToolCategory::Wiki,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }
    }

    mod action_override {
        use super::*;

        #[test]
        fn test_action_allow_overrides_restrictive_base() {
            let mut config = config_with_level(AccessLevel::Read);
            config
                .actions
                .insert("create_pipeline".to_string(), ActionPermission::Allow);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "retry_pipeline_job",
                        ToolCategory::Pipelines,
                        OperationType::Execute,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_action_deny_overrides_permissive_base() {
            let mut config = config_with_level(AccessLevel::Full);
            config
                .actions
                .insert("delete_project".to_string(), ActionPermission::Deny);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_project",
                        ToolCategory::Projects,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_action_overrides_category() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "pipelines".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![".*".to_string()],
                    allow: vec![],
                },
            );
            config
                .actions
                .insert("list_pipelines".to_string(), ActionPermission::Allow);
            let resolver = AccessResolver::new(&config).unwrap();

            // Category denies, but action override allows
            assert!(
                resolver
                    .check(
                        "list_pipelines",
                        ToolCategory::Pipelines,
                        OperationType::Read,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_action_deny_overrides_category_full() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            config
                .actions
                .insert("delete_issue".to_string(), ActionPermission::Deny);
            let resolver = AccessResolver::new(&config).unwrap();

            // Category says full, but action override denies
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_multiple_action_overrides() {
            let mut config = config_with_level(AccessLevel::Read);
            config
                .actions
                .insert("create_pipeline".to_string(), ActionPermission::Allow);
            config
                .actions
                .insert("retry_pipeline_job".to_string(), ActionPermission::Allow);
            config
                .actions
                .insert("delete_issue".to_string(), ActionPermission::Allow);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "retry_pipeline_job",
                        ToolCategory::Pipelines,
                        OperationType::Execute,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            // Non-overridden action still uses base
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_action_overrides_global_pattern() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_.*".to_string()];
            config
                .actions
                .insert("delete_issue_note".to_string(), ActionPermission::Allow);
            let resolver = AccessResolver::new(&config).unwrap();

            // Global pattern denies all deletes, but action override allows this one
            assert!(
                resolver
                    .check(
                        "delete_issue_note",
                        ToolCategory::IssueNotes,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
        }
    }

    mod project_override {
        use super::*;

        #[test]
        fn test_project_base_overrides_global_base() {
            let mut config = config_with_level(AccessLevel::Full);
            config.projects.insert(
                "prod/app".to_string(),
                ProjectAccessConfig {
                    all: Some(AccessLevel::Read),
                    deny: vec![],
                    allow: vec![],
                    categories: HashMap::new(),
                    actions: HashMap::new(),
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            // With project context, should be read-only
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("prod/app")
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        Some("prod/app")
                    )
                    .is_denied()
            );
            // Without project context, should be full
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_project_patterns_override_global() {
            let mut config = config_with_level(AccessLevel::Full);
            config.projects.insert(
                "prod/app".to_string(),
                ProjectAccessConfig {
                    all: None,
                    deny: vec!["delete_.*".to_string()],
                    allow: vec![],
                    categories: HashMap::new(),
                    actions: HashMap::new(),
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        Some("prod/app")
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        Some("dev/app")
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_project_category_overrides_global_category() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![".*".to_string()], // Use deny to actually block
                    allow: vec![],
                },
            );
            config
                .projects
                .insert("restricted/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Global category is full, but project restricts
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("restricted/repo")
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("other/repo")
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_project_action_overrides_global_action() {
            let mut config = config_with_level(AccessLevel::Read);
            config
                .actions
                .insert("create_pipeline".to_string(), ActionPermission::Deny);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config
                .actions
                .insert("create_pipeline".to_string(), ActionPermission::Allow);
            config.projects.insert("ci/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Global action denies, but project allows
            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        Some("ci/repo")
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_pipeline",
                        ToolCategory::Pipelines,
                        OperationType::Write,
                        Some("other/repo")
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_project_action_highest_priority() {
            let mut config = config_with_level(AccessLevel::Full);
            config.deny = vec!["delete_.*".to_string()];
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec!["delete_issue".to_string()],
                    allow: vec![],
                },
            );
            config
                .actions
                .insert("delete_issue".to_string(), ActionPermission::Deny);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config
                .actions
                .insert("delete_issue".to_string(), ActionPermission::Allow);
            config
                .projects
                .insert("special/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Everything says deny, but project action allows
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        Some("special/repo")
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_different_projects_different_access() {
            let mut config = config_with_level(AccessLevel::Read);

            let mut prod_config = ProjectAccessConfig::default();
            prod_config.all = Some(AccessLevel::None);
            config.projects.insert("prod/app".to_string(), prod_config);

            let mut dev_config = ProjectAccessConfig::default();
            dev_config.all = Some(AccessLevel::Full);
            config.projects.insert("dev/app".to_string(), dev_config);

            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("prod/app")
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        Some("dev/app")
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("other/app")
                    )
                    .is_allowed()
            );
        }
    }

    mod precedence {
        use super::*;

        #[test]
        fn test_project_action_beats_project_category() {
            let mut config = config_with_level(AccessLevel::Read);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::None,
                    deny: vec![".*".to_string()],
                    allow: vec![],
                },
            );
            proj_config
                .actions
                .insert("list_issues".to_string(), ActionPermission::Allow);
            config.projects.insert("test/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "list_issues",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("test/repo")
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "get_issue",
                        ToolCategory::Issues,
                        OperationType::Read,
                        Some("test/repo")
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_global_action_beats_project_category() {
            // Global action has higher priority than project category
            let mut config = config_with_level(AccessLevel::Full);
            config
                .actions
                .insert("create_issue".to_string(), ActionPermission::Deny);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            config.projects.insert("test/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Global action denies, project category would allow, but global action wins
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        Some("test/repo")
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_global_action_beats_global_category() {
            let mut config = config_with_level(AccessLevel::Read);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            config
                .actions
                .insert("delete_issue".to_string(), ActionPermission::Deny);
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_denied()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_global_category_beats_global_base() {
            let mut config = config_with_level(AccessLevel::None);
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            let resolver = AccessResolver::new(&config).unwrap();

            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        None
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "list_projects",
                        ToolCategory::Projects,
                        OperationType::Read,
                        None
                    )
                    .is_denied()
            );
        }

        #[test]
        fn test_project_category_beats_global_base() {
            // Project category should override global base
            let mut config = config_with_level(AccessLevel::Read);
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec![],
                    allow: vec![],
                },
            );
            config.projects.insert("test/repo".to_string(), proj_config);
            let resolver = AccessResolver::new(&config).unwrap();

            // Project category grants full access
            assert!(
                resolver
                    .check(
                        "delete_issue",
                        ToolCategory::Issues,
                        OperationType::Delete,
                        Some("test/repo")
                    )
                    .is_allowed()
            );
        }

        #[test]
        fn test_full_hierarchy_precedence() {
            let mut config = config_with_level(AccessLevel::Read);
            // Global base: read
            config.deny = vec!["delete_.*".to_string()]; // Global deny
            config.allow = vec!["delete_issue_note".to_string()]; // Global allow
            config.categories.insert(
                "issues".to_string(),
                CategoryAccessConfig {
                    level: AccessLevel::Full,
                    deny: vec!["delete_issue".to_string()],
                    allow: vec![],
                },
            );
            config
                .actions
                .insert("create_issue".to_string(), ActionPermission::Deny);

            let mut proj_config = ProjectAccessConfig::default();
            proj_config.all = Some(AccessLevel::Full);
            proj_config
                .actions
                .insert("create_issue".to_string(), ActionPermission::Allow);
            config
                .projects
                .insert("special/repo".to_string(), proj_config);

            let resolver = AccessResolver::new(&config).unwrap();

            // create_issue globally denied by action, but project allows
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        Some("special/repo")
                    )
                    .is_allowed()
            );
            assert!(
                resolver
                    .check(
                        "create_issue",
                        ToolCategory::Issues,
                        OperationType::Write,
                        None
                    )
                    .is_denied()
            );
        }
    }
}

// =============================================================================
// 4. Category Tests (42 tests - 21 categories × 2)
// =============================================================================

mod category_tests {
    use super::*;

    macro_rules! category_test {
        ($name_allowed:ident, $name_denied:ident, $category:ident, $read_tool:expr, $write_tool:expr) => {
            #[test]
            fn $name_allowed() {
                let mut config = config_with_level(AccessLevel::Read);
                config.categories.insert(
                    ToolCategory::$category.as_str().to_string(),
                    CategoryAccessConfig {
                        level: AccessLevel::Full,
                        deny: vec![],
                        allow: vec![],
                    },
                );
                let resolver = AccessResolver::new(&config).unwrap();
                assert!(
                    resolver
                        .check(
                            $write_tool,
                            ToolCategory::$category,
                            OperationType::Write,
                            None
                        )
                        .is_allowed()
                );
            }

            #[test]
            fn $name_denied() {
                // Use deny pattern to actually block (level=None just falls through)
                let mut config = config_with_level(AccessLevel::Full);
                config.categories.insert(
                    ToolCategory::$category.as_str().to_string(),
                    CategoryAccessConfig {
                        level: AccessLevel::None,
                        deny: vec![".*".to_string()],
                        allow: vec![],
                    },
                );
                let resolver = AccessResolver::new(&config).unwrap();
                assert!(
                    resolver
                        .check(
                            $read_tool,
                            ToolCategory::$category,
                            OperationType::Read,
                            None
                        )
                        .is_denied()
                );
            }
        };
    }

    category_test!(
        test_issues_allowed,
        test_issues_denied,
        Issues,
        "list_issues",
        "create_issue"
    );
    category_test!(
        test_issue_links_allowed,
        test_issue_links_denied,
        IssueLinks,
        "list_issue_links",
        "create_issue_link"
    );
    category_test!(
        test_issue_notes_allowed,
        test_issue_notes_denied,
        IssueNotes,
        "list_issue_notes",
        "create_issue_note"
    );
    category_test!(
        test_merge_requests_allowed,
        test_merge_requests_denied,
        MergeRequests,
        "list_merge_requests",
        "create_merge_request"
    );
    category_test!(
        test_mr_discussions_allowed,
        test_mr_discussions_denied,
        MrDiscussions,
        "list_mr_discussions",
        "create_mr_discussion"
    );
    category_test!(
        test_mr_drafts_allowed,
        test_mr_drafts_denied,
        MrDrafts,
        "list_mr_draft_notes",
        "create_mr_draft_note"
    );
    category_test!(
        test_repository_allowed,
        test_repository_denied,
        Repository,
        "get_file",
        "create_file"
    );
    category_test!(
        test_branches_allowed,
        test_branches_denied,
        Branches,
        "list_branches",
        "create_branch"
    );
    category_test!(
        test_commits_allowed,
        test_commits_denied,
        Commits,
        "list_commits",
        "create_commit"
    );
    category_test!(
        test_projects_allowed,
        test_projects_denied,
        Projects,
        "list_projects",
        "create_project"
    );
    category_test!(
        test_namespaces_allowed,
        test_namespaces_denied,
        Namespaces,
        "list_namespaces",
        "get_namespace"
    );
    category_test!(
        test_labels_allowed,
        test_labels_denied,
        Labels,
        "list_labels",
        "create_label"
    );
    category_test!(
        test_wiki_allowed,
        test_wiki_denied,
        Wiki,
        "list_wiki_pages",
        "create_wiki_page"
    );
    category_test!(
        test_pipelines_allowed,
        test_pipelines_denied,
        Pipelines,
        "list_pipelines",
        "create_pipeline"
    );
    category_test!(
        test_milestones_allowed,
        test_milestones_denied,
        Milestones,
        "list_milestones",
        "create_milestone"
    );
    category_test!(
        test_releases_allowed,
        test_releases_denied,
        Releases,
        "list_releases",
        "create_release"
    );
    category_test!(
        test_users_allowed,
        test_users_denied,
        Users,
        "list_users",
        "get_current_user"
    );
    category_test!(
        test_groups_allowed,
        test_groups_denied,
        Groups,
        "list_groups",
        "get_group"
    );
    category_test!(
        test_tags_allowed,
        test_tags_denied,
        Tags,
        "list_tags",
        "create_tag"
    );
    category_test!(
        test_search_allowed,
        test_search_denied,
        Search,
        "search_global",
        "search_project"
    );
}

// =============================================================================
// 5. Project-Specific Tests (16 tests)
// =============================================================================

mod project_specific_tests {
    use super::*;

    #[test]
    fn test_tool_allowed_in_specific_project() {
        let mut config = config_with_level(AccessLevel::None);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("allowed/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("allowed/repo")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_tool_denied_in_specific_project() {
        let mut config = config_with_level(AccessLevel::Full);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::None);
        config
            .projects
            .insert("denied/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("denied/repo")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_tool_allowed_globally_allowed_in_project() {
        let mut config = config_with_level(AccessLevel::Full);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config.projects.insert("test/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("test/repo")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_project_grants_access_when_global_denies() {
        let mut config = config_with_level(AccessLevel::None);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("exception/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("exception/repo")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_project_denies_when_global_allows() {
        let mut config = config_with_level(AccessLevel::Full);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::None);
        config
            .projects
            .insert("restricted/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("restricted/repo")
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_no_project_context_uses_global() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config.projects.insert("test/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // No project context should use global (read-only)
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_unknown_project_uses_global() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("known/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Unknown project should use global
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("unknown/repo")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_multiple_projects_independent() {
        let mut config = config_with_level(AccessLevel::Read);

        let mut prod_config = ProjectAccessConfig::default();
        prod_config.all = Some(AccessLevel::None);
        config.projects.insert("prod/app".to_string(), prod_config);

        let mut dev_config = ProjectAccessConfig::default();
        dev_config.all = Some(AccessLevel::Full);
        config.projects.insert("dev/app".to_string(), dev_config);

        let mut staging_config = ProjectAccessConfig::default();
        staging_config.all = Some(AccessLevel::Read);
        config
            .projects
            .insert("staging/app".to_string(), staging_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Each project has its own rules
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("prod/app")
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("dev/app")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("staging/app")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("staging/app")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_project_with_category_override() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        config.projects.insert("test/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("test/repo")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_project",
                    ToolCategory::Projects,
                    OperationType::Write,
                    Some("test/repo")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_project_with_action_override() {
        let mut config = config_with_level(AccessLevel::None);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config
            .actions
            .insert("list_issues".to_string(), ActionPermission::Allow);
        config.projects.insert("test/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("test/repo")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "get_issue",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("test/repo")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_project_pattern_deny() {
        let mut config = config_with_level(AccessLevel::Full);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.deny = vec!["delete_.*".to_string()];
        config
            .projects
            .insert("protected/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("protected/repo")
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("other/repo")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_project_pattern_allow() {
        let mut config = config_with_level(AccessLevel::None);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.allow = vec!["^list_".to_string()];
        config
            .projects
            .insert("readonly/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("readonly/repo")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("readonly/repo")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_nested_project_path() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("org/team/subteam/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("org/team/subteam/repo")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_project_with_special_chars() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("my-org/my_project.name".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("my-org/my_project.name")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_has_project_specific_access() {
        let mut config = config_with_level(AccessLevel::None);
        // Tool globally denied, but allowed in one project
        let mut proj_config = ProjectAccessConfig::default();
        proj_config
            .actions
            .insert("list_issues".to_string(), ActionPermission::Allow);
        config
            .projects
            .insert("special/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Has project-specific access
        assert!(resolver.has_project_specific_access("list_issues", ToolCategory::Issues));

        // Globally denied
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
        // But allowed in specific project
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("special/repo")
                )
                .is_allowed()
        );
    }
}

// =============================================================================
// 6. Complex Multi-Level Scenarios (12 tests)
// =============================================================================

mod complex_scenarios {
    use super::*;

    #[test]
    fn test_global_deny_category_allow_project_deny() {
        let mut config = config_with_level(AccessLevel::Full);
        config.deny = vec!["delete_issue".to_string()];
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec!["delete_issue".to_string()],
            },
        );

        // To actually deny at project level, we need a project-specific category override
        // because project base patterns (step 5) are checked AFTER global category (step 4)
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::None,
                deny: vec!["delete_issue".to_string()],
                allow: vec![],
            },
        );
        config
            .projects
            .insert("strict/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Project category deny overrides global category allow
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("strict/repo")
                )
                .is_denied()
        );
        // But globally, category allow should apply
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_global_allow_category_deny_action_allow() {
        let mut config = config_with_level(AccessLevel::Full);
        config.categories.insert(
            "pipelines".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::None,
                deny: vec![".*".to_string()],
                allow: vec![],
            },
        );
        config
            .actions
            .insert("list_pipelines".to_string(), ActionPermission::Allow);
        let resolver = AccessResolver::new(&config).unwrap();

        // Action allow should override category deny
        assert!(
            resolver
                .check(
                    "list_pipelines",
                    ToolCategory::Pipelines,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_all_six_levels_configured() {
        let mut config = AccessControlConfig {
            all: AccessLevel::None,                 // Level 6: Global base
            deny: vec![".*".to_string()],           // Level 5: Global deny (deny all)
            allow: vec!["list_issues".to_string()], // Level 4: Global allow (allow list_issues)
            categories: HashMap::new(),
            actions: HashMap::new(),
            projects: HashMap::new(),
        };
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,              // Level 3: Category
                deny: vec!["list_issues".to_string()], // Deny list_issues at category
                allow: vec![],
            },
        );
        config
            .actions
            .insert("list_issues".to_string(), ActionPermission::Deny); // Level 2: Global action
        let mut proj_config = ProjectAccessConfig::default();
        proj_config
            .actions
            .insert("list_issues".to_string(), ActionPermission::Allow); // Level 1: Project action
        config
            .projects
            .insert("exception/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Project action (highest priority) should allow
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("exception/repo")
                )
                .is_allowed()
        );
        // Without project, global action deny applies
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_global_pattern_category_pattern_project_pattern() {
        let mut config = config_with_level(AccessLevel::Full);
        config.deny = vec!["^delete_".to_string()];
        config.categories.insert(
            "issue_notes".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec!["delete_issue_note".to_string()],
            },
        );

        // To deny at project level, we need project-specific category override
        // because project base patterns (step 5) are checked AFTER global category (step 4)
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.categories.insert(
            "issue_notes".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::None,
                deny: vec!["delete_issue_note".to_string()],
                allow: vec![],
            },
        );
        config
            .projects
            .insert("no-notes/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Project category denies delete_issue_note
        assert!(
            resolver
                .check(
                    "delete_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Delete,
                    Some("no-notes/repo")
                )
                .is_denied()
        );
        // Without project, category allow applies
        assert!(
            resolver
                .check(
                    "delete_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_action_override_in_one_project_not_another() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_a = ProjectAccessConfig::default();
        proj_a
            .actions
            .insert("create_pipeline".to_string(), ActionPermission::Allow);
        config.projects.insert("proj-a".to_string(), proj_a);

        let proj_b = ProjectAccessConfig::default();
        config.projects.insert("proj-b".to_string(), proj_b);

        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "create_pipeline",
                    ToolCategory::Pipelines,
                    OperationType::Write,
                    Some("proj-a")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_pipeline",
                    ToolCategory::Pipelines,
                    OperationType::Write,
                    Some("proj-b")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_category_override_in_one_project_not_another() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_a = ProjectAccessConfig::default();
        proj_a.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        config.projects.insert("proj-a".to_string(), proj_a);

        let proj_b = ProjectAccessConfig::default();
        config.projects.insert("proj-b".to_string(), proj_b);

        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("proj-a")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("proj-b")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_conflicting_patterns_at_different_levels() {
        let mut config = config_with_level(AccessLevel::Full);
        config.deny = vec!["create_.*".to_string()];
        config.allow = vec!["create_issue".to_string()];
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec!["create_issue".to_string()],
                allow: vec![],
            },
        );
        let resolver = AccessResolver::new(&config).unwrap();

        // Global allow should win over global deny, but category pattern takes precedence
        // Category is checked before global patterns in hierarchy
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_read_only_production_full_access_dev() {
        let mut config = config_with_level(AccessLevel::Read);

        let mut prod_config = ProjectAccessConfig::default();
        prod_config.all = Some(AccessLevel::Read);
        prod_config.deny = vec![".*".to_string()];
        prod_config.allow = vec!["^list_".to_string(), "^get_".to_string()];
        config
            .projects
            .insert("company/production".to_string(), prod_config);

        let mut dev_config = ProjectAccessConfig::default();
        dev_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("company/development".to_string(), dev_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Production: only list/get allowed
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("company/production")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("company/production")
                )
                .is_denied()
        );

        // Development: full access
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("company/development")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_wiki_only_documentation_project() {
        let mut config = config_with_level(AccessLevel::None);

        let mut docs_config = ProjectAccessConfig::default();
        docs_config.categories.insert(
            "wiki".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        config
            .projects
            .insert("company/docs".to_string(), docs_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Only wiki operations allowed in docs project
        assert!(
            resolver
                .check(
                    "create_wiki_page",
                    ToolCategory::Wiki,
                    OperationType::Write,
                    Some("company/docs")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("company/docs")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_ci_management_project() {
        let mut config = config_with_level(AccessLevel::Read);

        let mut ci_config = ProjectAccessConfig::default();
        ci_config.categories.insert(
            "pipelines".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        ci_config.categories.insert(
            "repository".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Read,
                deny: vec![],
                allow: vec![],
            },
        );
        config
            .projects
            .insert("ci/automation".to_string(), ci_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Pipelines: full access
        assert!(
            resolver
                .check(
                    "create_pipeline",
                    ToolCategory::Pipelines,
                    OperationType::Write,
                    Some("ci/automation")
                )
                .is_allowed()
        );
        // Repository: read only
        assert!(
            resolver
                .check(
                    "create_file",
                    ToolCategory::Repository,
                    OperationType::Write,
                    Some("ci/automation")
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "get_file",
                    ToolCategory::Repository,
                    OperationType::Read,
                    Some("ci/automation")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_sandbox_with_no_deletes() {
        let mut config = config_with_level(AccessLevel::Read);

        let mut sandbox_config = ProjectAccessConfig::default();
        sandbox_config.all = Some(AccessLevel::Full);
        sandbox_config.deny = vec!["^delete_".to_string()];
        config
            .projects
            .insert("sandbox/playground".to_string(), sandbox_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Full access except deletes
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("sandbox/playground")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("sandbox/playground")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_different_access_per_category_per_project() {
        let mut config = config_with_level(AccessLevel::Read);

        let mut proj_config = ProjectAccessConfig::default();
        proj_config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        proj_config.categories.insert(
            "merge_requests".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec!["merge_merge_request".to_string()],
                allow: vec![],
            },
        );
        proj_config.categories.insert(
            "pipelines".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Read,
                deny: vec![],
                allow: vec!["retry_pipeline_job".to_string()],
            },
        );
        config
            .projects
            .insert("team/project".to_string(), proj_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Issues: full
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("team/project")
                )
                .is_allowed()
        );
        // MRs: full except merge
        assert!(
            resolver
                .check(
                    "create_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Write,
                    Some("team/project")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "merge_merge_request",
                    ToolCategory::MergeRequests,
                    OperationType::Execute,
                    Some("team/project")
                )
                .is_denied()
        );
        // Pipelines: read + retry
        assert!(
            resolver
                .check(
                    "list_pipelines",
                    ToolCategory::Pipelines,
                    OperationType::Read,
                    Some("team/project")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "retry_pipeline_job",
                    ToolCategory::Pipelines,
                    OperationType::Execute,
                    Some("team/project")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "create_pipeline",
                    ToolCategory::Pipelines,
                    OperationType::Write,
                    Some("team/project")
                )
                .is_denied()
        );
    }
}

// =============================================================================
// 7. Error Handling Tests (8 tests)
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_check_returns_denied_with_reason() {
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();

        let result = resolver.check(
            "create_issue",
            ToolCategory::Issues,
            OperationType::Write,
            None,
        );
        assert!(result.is_denied());
    }

    #[test]
    fn test_allow_all_resolver() {
        let resolver = AccessResolver::allow_all();

        assert!(
            resolver
                .check(
                    "delete_project",
                    ToolCategory::Projects,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "any_tool",
                    ToolCategory::Issues,
                    OperationType::Execute,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_deny_all_resolver() {
        let resolver = AccessResolver::deny_all();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
        assert!(
            resolver
                .check("any_tool", ToolCategory::Issues, OperationType::Read, None)
                .is_denied()
        );
    }

    #[test]
    fn test_none_level_denies_all() {
        // AccessLevel::None denies all operations
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();

        // None level means everything should be denied
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_unknown_category_in_config_rejected() {
        let mut config = config_with_level(AccessLevel::Read);
        // Add a category that doesn't exist - should cause error
        config.categories.insert(
            "nonexistent_category".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        let result = AccessResolver::new(&config);

        // Should fail because category is unknown
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_project_path_uses_global() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config.projects.insert("test/repo".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Empty string should be treated as no project (uses global)
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    Some("")
                )
                .is_denied()
        );
    }

    #[test]
    fn test_is_allowed_is_denied_mutually_exclusive() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();

        let allowed_result = resolver.check(
            "list_issues",
            ToolCategory::Issues,
            OperationType::Read,
            None,
        );
        assert!(allowed_result.is_allowed());
        assert!(!allowed_result.is_denied());

        let denied_result = resolver.check(
            "create_issue",
            ToolCategory::Issues,
            OperationType::Write,
            None,
        );
        assert!(denied_result.is_denied());
        assert!(!denied_result.is_allowed());
    }

    #[test]
    fn test_resolver_new_handles_config() {
        // Test that new() properly initializes with different configs
        let configs = vec![
            config_with_level(AccessLevel::None),
            config_with_level(AccessLevel::Read),
            config_with_level(AccessLevel::Full),
        ];

        for config in configs {
            let resolver = AccessResolver::new(&config);
            assert!(resolver.is_ok());
        }
    }
}

// =============================================================================
// 8. Edge Case Tests (12 tests)
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_none_level_config_denies_all() {
        // AccessLevel::None denies all operations
        let config = config_with_level(AccessLevel::None);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "create_issue",
                    ToolCategory::Issues,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_all_categories_have_consistent_behavior() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();

        for category in ToolCategory::all() {
            // All categories should allow read, deny write
            assert!(
                resolver
                    .check("list_test", *category, OperationType::Read, None)
                    .is_allowed()
            );
            assert!(
                resolver
                    .check("create_test", *category, OperationType::Write, None)
                    .is_denied()
            );
        }
    }

    #[test]
    fn test_very_long_tool_name() {
        let mut config = config_with_level(AccessLevel::None);
        let long_name = "a".repeat(1000);
        config.allow = vec![long_name.clone()];
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(&long_name, ToolCategory::Issues, OperationType::Write, None)
                .is_allowed()
        );
    }

    #[test]
    fn test_unicode_in_project_path() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config.projects.insert("组织/项目".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("组织/项目")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_many_projects_configured() {
        let mut config = config_with_level(AccessLevel::Read);

        // Add 100 projects
        for i in 0..100 {
            let mut proj_config = ProjectAccessConfig::default();
            proj_config.all = Some(if i % 2 == 0 {
                AccessLevel::Full
            } else {
                AccessLevel::None
            });
            config
                .projects
                .insert(format!("org/project-{}", i), proj_config);
        }

        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("org/project-0")
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    Some("org/project-1")
                )
                .is_denied()
        );
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("org/project-98")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_many_patterns_configured() {
        let mut config = config_with_level(AccessLevel::None);

        // Add 100 allow patterns
        for i in 0..100 {
            config.allow.push(format!("^tool_{}$", i));
        }

        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check("tool_0", ToolCategory::Issues, OperationType::Write, None)
                .is_allowed()
        );
        assert!(
            resolver
                .check("tool_99", ToolCategory::Issues, OperationType::Write, None)
                .is_allowed()
        );
        assert!(
            resolver
                .check("tool_100", ToolCategory::Issues, OperationType::Write, None)
                .is_denied()
        );
    }

    #[test]
    fn test_pattern_with_special_regex_chars() {
        let mut config = config_with_level(AccessLevel::None);
        // Test that special regex chars work properly
        config.allow = vec![r"list\[issues\]".to_string()];
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "list[issues]",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "list_issues",
                    ToolCategory::Issues,
                    OperationType::Read,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_deeply_nested_project_path() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("a/b/c/d/e/f/g/h/i/j".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("a/b/c/d/e/f/g/h/i/j")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_project_path_with_dots_and_dashes() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("my-org.name/my_project-v2.0".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("my-org.name/my_project-v2.0")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_same_tool_different_categories() {
        // This tests that category is properly considered
        let mut config = config_with_level(AccessLevel::Read);
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec![],
                allow: vec![],
            },
        );
        let resolver = AccessResolver::new(&config).unwrap();

        // Same operation name, different categories
        assert!(
            resolver
                .check(
                    "delete_test",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
        assert!(
            resolver
                .check(
                    "delete_test",
                    ToolCategory::Projects,
                    OperationType::Delete,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_all_operation_types_respected() {
        let config = config_with_level(AccessLevel::Read);
        let resolver = AccessResolver::new(&config).unwrap();

        // Only Read should be allowed
        assert!(
            resolver
                .check("op", ToolCategory::Issues, OperationType::Read, None)
                .is_allowed()
        );
        assert!(
            resolver
                .check("op", ToolCategory::Issues, OperationType::Write, None)
                .is_denied()
        );
        assert!(
            resolver
                .check("op", ToolCategory::Issues, OperationType::Delete, None)
                .is_denied()
        );
        assert!(
            resolver
                .check("op", ToolCategory::Issues, OperationType::Execute, None)
                .is_denied()
        );
    }

    #[test]
    fn test_case_sensitivity_in_project_paths() {
        let mut config = config_with_level(AccessLevel::Read);
        let mut proj_config = ProjectAccessConfig::default();
        proj_config.all = Some(AccessLevel::Full);
        config
            .projects
            .insert("MyOrg/MyProject".to_string(), proj_config);
        let resolver = AccessResolver::new(&config).unwrap();

        // Exact case should match
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("MyOrg/MyProject")
                )
                .is_allowed()
        );
        // Different case should not match (uses global)
        assert!(
            resolver
                .check(
                    "delete_issue",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    Some("myorg/myproject")
                )
                .is_denied()
        );
    }
}
