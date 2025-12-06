//! Access control integration tests

use tanuki_mcp::access_control::{AccessResolver, OperationType, ToolCategory};
use tanuki_mcp::config::{
    AccessControlConfig, AccessLevel, ActionPermission, CategoryAccessConfig,
};
use std::collections::HashMap;

fn create_test_config() -> AccessControlConfig {
    AccessControlConfig {
        all: AccessLevel::Read,
        deny: vec![],
        allow: vec![],
        categories: HashMap::new(),
        actions: HashMap::new(),
        projects: HashMap::new(),
    }
}

#[test]
fn test_base_access_level_read() {
    let config = create_test_config();
    let resolver = AccessResolver::new(&config).unwrap();

    // Read operations should be allowed
    assert!(resolver
        .check(
            "list_issues",
            ToolCategory::Issues,
            OperationType::Read,
            None
        )
        .is_allowed());
    assert!(resolver
        .check("get_issue", ToolCategory::Issues, OperationType::Read, None)
        .is_allowed());

    // Write operations should be denied
    assert!(resolver
        .check(
            "create_issue",
            ToolCategory::Issues,
            OperationType::Write,
            None
        )
        .is_denied());

    // Delete operations should be denied
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_denied());
}

#[test]
fn test_base_access_level_full() {
    let mut config = create_test_config();
    config.all = AccessLevel::Full;
    let resolver = AccessResolver::new(&config).unwrap();

    // All operations should be allowed
    assert!(resolver
        .check(
            "list_issues",
            ToolCategory::Issues,
            OperationType::Read,
            None
        )
        .is_allowed());
    assert!(resolver
        .check(
            "create_issue",
            ToolCategory::Issues,
            OperationType::Write,
            None
        )
        .is_allowed());
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_allowed());
}

#[test]
fn test_category_override() {
    let mut config = create_test_config();
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
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_allowed());

    // Other categories should still be read-only
    assert!(resolver
        .check(
            "create_project",
            ToolCategory::Projects,
            OperationType::Write,
            None
        )
        .is_denied());
}

#[test]
fn test_category_deny_pattern() {
    let mut config = create_test_config();
    config.all = AccessLevel::Full;
    config.categories.insert(
        "merge_requests".to_string(),
        CategoryAccessConfig {
            level: AccessLevel::Full,
            deny: vec!["merge_merge_request".to_string()],
            allow: vec![],
        },
    );
    let resolver = AccessResolver::new(&config).unwrap();

    // Regular MR operations should work
    assert!(resolver
        .check(
            "create_merge_request",
            ToolCategory::MergeRequests,
            OperationType::Write,
            None
        )
        .is_allowed());

    // Merge should be denied
    assert!(resolver
        .check(
            "merge_merge_request",
            ToolCategory::MergeRequests,
            OperationType::Write,
            None
        )
        .is_denied());
}

#[test]
fn test_action_override_allow() {
    let mut config = create_test_config();
    config
        .actions
        .insert("create_pipeline".to_string(), ActionPermission::Allow);
    let resolver = AccessResolver::new(&config).unwrap();

    // Base is read, but create_pipeline should be explicitly allowed
    assert!(resolver
        .check(
            "create_pipeline",
            ToolCategory::Pipelines,
            OperationType::Write,
            None
        )
        .is_allowed());

    // Other write operations should still be denied
    assert!(resolver
        .check(
            "retry_pipeline_job",
            ToolCategory::Pipelines,
            OperationType::Write,
            None
        )
        .is_denied());
}

#[test]
fn test_action_override_deny() {
    let mut config = create_test_config();
    config.all = AccessLevel::Full;
    config
        .actions
        .insert("delete_project".to_string(), ActionPermission::Deny);
    let resolver = AccessResolver::new(&config).unwrap();

    // delete_project should be denied even with full access
    assert!(resolver
        .check(
            "delete_project",
            ToolCategory::Projects,
            OperationType::Delete,
            None
        )
        .is_denied());

    // Other deletes should work
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_allowed());
}

#[test]
fn test_global_deny_pattern() {
    let mut config = create_test_config();
    config.all = AccessLevel::Full;
    config.deny = vec!["delete_.*".to_string()];
    let resolver = AccessResolver::new(&config).unwrap();

    // All delete operations should be denied
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_denied());
    assert!(resolver
        .check(
            "delete_project",
            ToolCategory::Projects,
            OperationType::Delete,
            None
        )
        .is_denied());

    // Non-delete operations should work
    assert!(resolver
        .check(
            "create_issue",
            ToolCategory::Issues,
            OperationType::Write,
            None
        )
        .is_allowed());
}

#[test]
fn test_global_allow_overrides_deny() {
    let mut config = create_test_config();
    config.all = AccessLevel::Full;
    config.deny = vec!["delete_.*".to_string()];
    config.allow = vec!["delete_issue".to_string()];
    let resolver = AccessResolver::new(&config).unwrap();

    // delete_issue should be allowed (allow overrides deny)
    assert!(resolver
        .check(
            "delete_issue",
            ToolCategory::Issues,
            OperationType::Delete,
            None
        )
        .is_allowed());

    // Other deletes should still be denied
    assert!(resolver
        .check(
            "delete_project",
            ToolCategory::Projects,
            OperationType::Delete,
            None
        )
        .is_denied());
}

#[test]
fn test_allow_all_resolver() {
    let resolver = AccessResolver::allow_all();

    // Everything should be allowed
    assert!(resolver
        .check(
            "delete_project",
            ToolCategory::Projects,
            OperationType::Delete,
            None
        )
        .is_allowed());
}

#[test]
fn test_deny_all_resolver() {
    let resolver = AccessResolver::deny_all();

    // Everything should be denied
    assert!(resolver
        .check(
            "list_issues",
            ToolCategory::Issues,
            OperationType::Read,
            None
        )
        .is_denied());
}
