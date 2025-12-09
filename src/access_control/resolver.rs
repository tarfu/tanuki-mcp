//! Access control resolver
//!
//! Implements hierarchical access control resolution with the following precedence
//! (highest to lowest):
//! 1. Project-specific action override
//! 2. Global action override
//! 3. Project-specific category (level + patterns)
//! 4. Global category (level + patterns)
//! 5. Project-specific base (all + patterns)
//! 6. Global base (all + patterns)

use crate::access_control::patterns::PatternMatcher;
use crate::access_control::types::{OperationType, ToolCategory};
use crate::config::{
    AccessControlConfig, AccessLevel, ActionPermission, CategoryAccessConfig, ProjectAccessConfig,
};
use crate::error::{AccessDeniedError, ConfigError};
use std::collections::HashMap;
use tracing::{debug, trace};

/// Access control resolver
///
/// Evaluates whether a tool call is permitted based on the hierarchical
/// access control configuration.
pub struct AccessResolver {
    /// Global base access level
    base_level: AccessLevel,
    /// Global deny patterns
    global_deny: PatternMatcher,
    /// Global allow patterns
    global_allow: PatternMatcher,
    /// Category configurations
    categories: HashMap<ToolCategory, CategoryConfig>,
    /// Individual action overrides
    actions: HashMap<String, ActionPermission>,
    /// Project-specific configurations
    projects: HashMap<String, ProjectConfig>,
}

/// Compiled category configuration
struct CategoryConfig {
    level: AccessLevel,
    deny: PatternMatcher,
    allow: PatternMatcher,
}

/// Compiled project configuration
struct ProjectConfig {
    base_level: Option<AccessLevel>,
    deny: PatternMatcher,
    allow: PatternMatcher,
    categories: HashMap<ToolCategory, CategoryConfig>,
    actions: HashMap<String, ActionPermission>,
}

/// Result of access check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDecision {
    /// Access is allowed
    Allowed,
    /// Access is denied with a reason
    Denied(String),
}

impl AccessDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessDecision::Allowed)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, AccessDecision::Denied(_))
    }
}

impl AccessResolver {
    /// Create a new resolver from configuration
    pub fn new(config: &AccessControlConfig) -> Result<Self, ConfigError> {
        // Compile global patterns
        let global_deny = PatternMatcher::new(&config.deny)?;
        let global_allow = PatternMatcher::new(&config.allow)?;

        // Compile category configurations
        let mut categories = HashMap::new();
        for (name, cat_config) in &config.categories {
            if let Some(category) = ToolCategory::try_parse(name) {
                categories.insert(category, Self::compile_category(cat_config)?);
            } else {
                return Err(ConfigError::Invalid {
                    message: format!("Unknown category: {}", name),
                });
            }
        }

        // Compile project configurations
        let mut projects = HashMap::new();
        for (name, proj_config) in &config.projects {
            projects.insert(name.clone(), Self::compile_project(proj_config)?);
        }

        Ok(Self {
            base_level: config.all,
            global_deny,
            global_allow,
            categories,
            actions: config.actions.clone(),
            projects,
        })
    }

    fn compile_category(config: &CategoryAccessConfig) -> Result<CategoryConfig, ConfigError> {
        Ok(CategoryConfig {
            level: config.level,
            deny: PatternMatcher::new(&config.deny)?,
            allow: PatternMatcher::new(&config.allow)?,
        })
    }

    fn compile_project(config: &ProjectAccessConfig) -> Result<ProjectConfig, ConfigError> {
        let mut categories = HashMap::new();
        for (name, cat_config) in &config.categories {
            if let Some(category) = ToolCategory::try_parse(name) {
                categories.insert(category, Self::compile_category(cat_config)?);
            }
        }

        Ok(ProjectConfig {
            base_level: config.all,
            deny: PatternMatcher::new(&config.deny)?,
            allow: PatternMatcher::new(&config.allow)?,
            categories,
            actions: config.actions.clone(),
        })
    }

    /// Check if a tool call is permitted
    ///
    /// Returns an AccessDecision indicating whether access is allowed or denied.
    pub fn check(
        &self,
        tool_name: &str,
        category: ToolCategory,
        operation: OperationType,
        project: Option<&str>,
    ) -> AccessDecision {
        debug!(
            tool = tool_name,
            category = %category,
            operation = %operation,
            project = ?project,
            "Checking access"
        );

        // 1. Check project-specific action override
        if let Some(proj_name) = project
            && let Some(proj_config) = self.projects.get(proj_name)
            && let Some(permission) = proj_config.actions.get(tool_name)
        {
            trace!("Matched project action override");
            return match permission {
                ActionPermission::Allow => AccessDecision::Allowed,
                ActionPermission::Deny => {
                    AccessDecision::Denied(format!("Explicitly denied for project '{}'", proj_name))
                }
            };
        }

        // 2. Check global action override
        if let Some(permission) = self.actions.get(tool_name) {
            trace!("Matched global action override");
            return match permission {
                ActionPermission::Allow => AccessDecision::Allowed,
                ActionPermission::Deny => {
                    AccessDecision::Denied("Explicitly denied by action override".to_string())
                }
            };
        }

        // 3. Check project-specific category
        if let Some(proj_name) = project
            && let Some(proj_config) = self.projects.get(proj_name)
            && let Some(cat_config) = proj_config.categories.get(&category)
            && let Some(decision) = self.check_level_and_patterns(tool_name, operation, cat_config)
        {
            trace!("Matched project category config");
            return decision;
        }

        // 4. Check global category
        if let Some(cat_config) = self.categories.get(&category)
            && let Some(decision) = self.check_level_and_patterns(tool_name, operation, cat_config)
        {
            trace!("Matched global category config");
            return decision;
        }

        // 5. Check project-specific base
        if let Some(proj_name) = project
            && let Some(proj_config) = self.projects.get(proj_name)
        {
            // Check project patterns first
            if let Some(pattern) = proj_config.allow.find_match(tool_name) {
                trace!("Matched project allow pattern: {}", pattern);
                return AccessDecision::Allowed;
            }
            if let Some(pattern) = proj_config.deny.find_match(tool_name) {
                trace!("Matched project deny pattern: {}", pattern);
                return AccessDecision::Denied(format!("Denied by project pattern '{}'", pattern));
            }

            // Check project base level
            if let Some(level) = proj_config.base_level {
                trace!("Using project base level: {:?}", level);
                return self.check_access_level(level, operation);
            }
        }

        // 6. Check global base
        // Check global patterns
        if let Some(pattern) = self.global_allow.find_match(tool_name) {
            trace!("Matched global allow pattern: {}", pattern);
            return AccessDecision::Allowed;
        }
        if let Some(pattern) = self.global_deny.find_match(tool_name) {
            trace!("Matched global deny pattern: {}", pattern);
            return AccessDecision::Denied(format!("Denied by pattern '{}'", pattern));
        }

        // Fall back to base level
        trace!("Using global base level: {:?}", self.base_level);
        self.check_access_level(self.base_level, operation)
    }

    /// Check level and patterns for a category config
    fn check_level_and_patterns(
        &self,
        tool_name: &str,
        operation: OperationType,
        config: &CategoryConfig,
    ) -> Option<AccessDecision> {
        // Allow patterns override deny patterns at the same level
        if config.allow.find_match(tool_name).is_some() {
            return Some(AccessDecision::Allowed);
        }
        if let Some(pattern) = config.deny.find_match(tool_name) {
            return Some(AccessDecision::Denied(format!(
                "Denied by category pattern '{}'",
                pattern
            )));
        }

        // If no pattern matched, check the level
        if config.level != AccessLevel::None {
            return Some(self.check_access_level(config.level, operation));
        }

        // No decision at this level
        None
    }

    /// Check if an access level permits an operation
    fn check_access_level(&self, level: AccessLevel, operation: OperationType) -> AccessDecision {
        match (level, operation) {
            (AccessLevel::Full, _) => AccessDecision::Allowed,
            (AccessLevel::Read, OperationType::Read) => AccessDecision::Allowed,
            (AccessLevel::Read, _) => AccessDecision::Denied(format!(
                "Operation '{}' requires write access, but only read access is granted",
                operation
            )),
            (AccessLevel::Deny, _) => {
                AccessDecision::Denied("Access explicitly denied at this level".to_string())
            }
            (AccessLevel::None, _) => {
                AccessDecision::Denied("No access granted at this level".to_string())
            }
        }
    }

    /// Check if a tool call is permitted, returning an error if denied
    pub fn require(
        &self,
        tool_name: &str,
        category: ToolCategory,
        operation: OperationType,
        project: Option<&str>,
    ) -> Result<(), AccessDeniedError> {
        match self.check(tool_name, category, operation, project) {
            AccessDecision::Allowed => Ok(()),
            AccessDecision::Denied(reason) => Err(AccessDeniedError::new(tool_name, reason)),
        }
    }

    /// Check if a tool is globally denied (denied everywhere, no project grants access).
    ///
    /// This is used to mark tools as "UNAVAILABLE" in the tool listing.
    /// Returns `true` only if:
    /// 1. Tool is explicitly denied globally (action override), AND no project allows it, OR
    /// 2. Tool is denied by global patterns AND no project allows it, OR
    /// 3. Global base level denies the operation AND no project/category grants access
    pub fn is_globally_denied(
        &self,
        tool_name: &str,
        category: ToolCategory,
        operation: OperationType,
    ) -> bool {
        // First check: is it denied globally without project context?
        let global_decision = self.check(tool_name, category, operation, None);

        if global_decision.is_allowed() {
            // If it's allowed globally, it's not globally denied
            return false;
        }

        // It's denied globally. Now check if ANY project grants access.
        for (proj_name, proj_config) in &self.projects {
            // Check project-specific action override
            if let Some(ActionPermission::Allow) = proj_config.actions.get(tool_name) {
                trace!(
                    tool = tool_name,
                    project = proj_name,
                    "Tool has project-specific allow override"
                );
                return false; // At least one project allows it
            }

            // Check project-specific category
            if let Some(cat_config) = proj_config.categories.get(&category) {
                // Check allow patterns
                if cat_config.allow.find_match(tool_name).is_some() {
                    return false;
                }
                // Check if category level would allow
                if cat_config.level != AccessLevel::None
                    && self
                        .check_access_level(cat_config.level, operation)
                        .is_allowed()
                {
                    return false;
                }
            }

            // Check project allow patterns
            if proj_config.allow.find_match(tool_name).is_some() {
                return false;
            }

            // Check project base level
            if let Some(level) = proj_config.base_level
                && self.check_access_level(level, operation).is_allowed()
            {
                return false;
            }
        }

        // No project grants access - tool is truly globally denied
        true
    }

    /// Check if a tool has project-specific restrictions.
    ///
    /// Returns `true` if the tool might be available for some projects but not others.
    /// Used to provide helpful error messages like "not allowed for this project, but may be available for others".
    pub fn has_project_specific_access(&self, tool_name: &str, category: ToolCategory) -> bool {
        // Check if any project has specific overrides for this tool or category
        for proj_config in self.projects.values() {
            if proj_config.actions.contains_key(tool_name) {
                return true;
            }
            if proj_config.categories.contains_key(&category) {
                return true;
            }
            if proj_config.base_level.is_some() {
                return true;
            }
            if !proj_config.allow.is_empty() || !proj_config.deny.is_empty() {
                return true;
            }
        }
        false
    }

    /// Create a permissive resolver that allows everything (for testing)
    pub fn allow_all() -> Self {
        Self {
            base_level: AccessLevel::Full,
            global_deny: PatternMatcher::empty(),
            global_allow: PatternMatcher::empty(),
            categories: HashMap::new(),
            actions: HashMap::new(),
            projects: HashMap::new(),
        }
    }

    /// Create a restrictive resolver that denies everything
    pub fn deny_all() -> Self {
        Self {
            base_level: AccessLevel::None,
            global_deny: PatternMatcher::empty(),
            global_allow: PatternMatcher::empty(),
            categories: HashMap::new(),
            actions: HashMap::new(),
            projects: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> AccessControlConfig {
        AccessControlConfig::default()
    }

    #[test]
    fn test_allow_all() {
        let resolver = AccessResolver::allow_all();

        assert!(
            resolver
                .check(
                    "any_tool",
                    ToolCategory::Issues,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_deny_all() {
        let resolver = AccessResolver::deny_all();

        assert!(
            resolver
                .check("any_tool", ToolCategory::Issues, OperationType::Read, None)
                .is_denied()
        );
    }

    #[test]
    fn test_base_level_read() {
        let mut config = default_config();
        config.all = AccessLevel::Read;
        let resolver = AccessResolver::new(&config).unwrap();

        // Read operations should be allowed
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

        // Write operations should be denied
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
    fn test_global_deny_pattern() {
        let mut config = default_config();
        config.all = AccessLevel::Full;
        config.deny = vec!["^delete_".to_string()];
        let resolver = AccessResolver::new(&config).unwrap();

        // Delete operations matching pattern should be denied
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

        // Other operations should be allowed
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
    fn test_allow_overrides_deny() {
        let mut config = default_config();
        config.all = AccessLevel::Full;
        config.deny = vec!["^delete_".to_string()];
        config.allow = vec!["^delete_label$".to_string()];
        let resolver = AccessResolver::new(&config).unwrap();

        // delete_label should be allowed because allow overrides deny
        assert!(
            resolver
                .check(
                    "delete_label",
                    ToolCategory::Labels,
                    OperationType::Delete,
                    None
                )
                .is_allowed()
        );

        // Other deletes should still be denied
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
    fn test_category_override() {
        let mut config = default_config();
        config.all = AccessLevel::Read;
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                ..Default::default()
            },
        );
        let resolver = AccessResolver::new(&config).unwrap();

        // Issues category should have full access
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

        // Other categories should only have read
        assert!(
            resolver
                .check(
                    "create_label",
                    ToolCategory::Labels,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );
    }

    #[test]
    fn test_action_override() {
        let mut config = default_config();
        config.all = AccessLevel::Read;
        config
            .actions
            .insert("create_issue_note".to_string(), ActionPermission::Allow);
        config
            .actions
            .insert("merge_merge_request".to_string(), ActionPermission::Deny);
        let resolver = AccessResolver::new(&config).unwrap();

        // Explicitly allowed action
        assert!(
            resolver
                .check(
                    "create_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Write,
                    None
                )
                .is_allowed()
        );

        // Explicitly denied action
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
    fn test_project_override() {
        let mut config = default_config();
        config.all = AccessLevel::Full;

        // Production project is read-only
        config.projects.insert(
            "prod/app".to_string(),
            ProjectAccessConfig {
                all: Some(AccessLevel::Read),
                ..Default::default()
            },
        );
        let resolver = AccessResolver::new(&config).unwrap();

        // Global access is full
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

        // But prod/app is read-only
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

        // Read is still allowed
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
    }

    #[test]
    fn test_project_action_override() {
        let mut config = default_config();
        config.all = AccessLevel::Read;

        // Allow create_issue_note only for staging project
        let mut proj_config = ProjectAccessConfig::default();
        proj_config
            .actions
            .insert("create_issue_note".to_string(), ActionPermission::Allow);
        config
            .projects
            .insert("staging/app".to_string(), proj_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Globally, create_issue_note is denied (read-only)
        assert!(
            resolver
                .check(
                    "create_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Write,
                    None
                )
                .is_denied()
        );

        // But allowed for staging
        assert!(
            resolver
                .check(
                    "create_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Write,
                    Some("staging/app")
                )
                .is_allowed()
        );
    }

    #[test]
    fn test_complex_config() {
        let mut config = default_config();
        config.all = AccessLevel::Read;
        config.deny = vec!["^delete_".to_string()];

        // Issues have full access
        config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Full,
                deny: vec!["^delete_issue$".to_string()],
                ..Default::default()
            },
        );

        // But merge_merge_request is explicitly denied
        config
            .actions
            .insert("merge_merge_request".to_string(), ActionPermission::Deny);

        // Production has a project-level category override for issues to be read-only
        // and an action override to allow notes
        let mut prod_config = ProjectAccessConfig {
            all: Some(AccessLevel::Read),
            ..Default::default()
        };
        // Add project-specific category to override global issues=full
        prod_config.categories.insert(
            "issues".to_string(),
            CategoryAccessConfig {
                level: AccessLevel::Read,
                deny: vec![],
                allow: vec![],
            },
        );
        prod_config
            .actions
            .insert("create_issue_note".to_string(), ActionPermission::Allow);
        config.projects.insert("prod/app".to_string(), prod_config);

        let resolver = AccessResolver::new(&config).unwrap();

        // Test various scenarios
        // 1. Read is always allowed globally
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

        // 2. Issues create is allowed (category override)
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

        // 3. Delete issue is denied (category pattern)
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

        // 4. Merge MR is always denied (action override)
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

        // 5. Prod issues are read-only (project category override)
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

        // 6. But notes are allowed on prod (action override)
        assert!(
            resolver
                .check(
                    "create_issue_note",
                    ToolCategory::IssueNotes,
                    OperationType::Write,
                    Some("prod/app")
                )
                .is_allowed()
        );
    }
}
