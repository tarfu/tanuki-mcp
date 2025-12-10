# Access Control

tanuki-mcp provides a fine-grained, hierarchical access control system that allows you to precisely control which GitLab operations are permitted.

## Overview

The access control system is designed around three core concepts:

1. **Access Levels** - Define the base permission level
2. **Operation Types** - Classify what kind of action a tool performs
3. **Hierarchical Resolution** - Multiple layers of configuration that override each other

## Access Levels

| Level | Description |
|-------|-------------|
| `none` | No decision at this level (falls through to next level in hierarchy) |
| `deny` | Explicitly deny all operations |
| `read` | Read-only operations (list, get, search) |
| `full` | Full access (all operations including create, update, delete, execute) |

## Operation Types

Each tool is classified by what type of operation it performs:

| Type | Description | Examples |
|------|-------------|----------|
| `read` | Retrieves data without modification | `list_issues`, `get_merge_request` |
| `write` | Creates or updates resources | `create_issue`, `update_merge_request` |
| `delete` | Removes resources | `delete_issue`, `delete_branch` |
| `execute` | Triggers actions | `merge_merge_request`, `retry_pipeline_job` |

## Tool Categories

Tools are organized into 20 categories:

| Category | Tools | Description |
|----------|-------|-------------|
| `issues` | 8 | Issue management |
| `issue_notes` | 5 | Issue comments |
| `issue_links` | 3 | Issue relationships |
| `merge_requests` | 8 | Merge request management |
| `mr_discussions` | 7 | MR comments and threads |
| `mr_drafts` | 7 | MR draft notes |
| `repository` | 7 | Files, tree, search |
| `branches` | 2 | Branch operations |
| `commits` | 3 | Commit operations |
| `projects` | 6 | Project management |
| `namespaces` | 3 | Namespace operations |
| `labels` | 5 | Label management |
| `wiki` | 5 | Wiki pages |
| `pipelines` | 12 | CI/CD pipelines |
| `milestones` | 9 | Milestone management |
| `releases` | 6 | Release management |
| `users` | 2 | User operations |
| `groups` | 2 | Group operations |
| `tags` | 9 | Git tag operations |
| `search` | 5 | Search operations |

## Hierarchical Resolution

Access is resolved in priority order (highest priority first):

```
1. Project-specific action override
2. Global action override
3. Project-specific category level
4. Global category level
5. Project-specific base (all)
6. Global base (all)
```

At each level, pattern matching is applied:
1. Check `deny` patterns first
2. Check `allow` patterns (can override deny at same level)

### Resolution Flow

```
                    ┌─────────────────────────────┐
                    │      Tool Request           │
                    │  (tool_name, project_path)  │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │  Project-specific action?   │
                    │  projects.{path}.actions    │
                    └──────────────┬──────────────┘
                          No       │     Yes
                    ┌──────────────┼──────────────┐
                    │              │              │
                    ▼              │              ▼
           ┌────────────────┐     │     ┌────────────────┐
           │ Global action? │     │     │ Return result  │
           │ actions.{tool} │     │     └────────────────┘
           └───────┬────────┘     │
              No   │   Yes        │
           ┌───────┼───────┐      │
           │       │       │      │
           ▼       │       ▼      │
    ┌──────────┐   │  ┌──────────┐│
    │ Project  │   │  │  Return  ││
    │ category │   │  │  result  ││
    └────┬─────┘   │  └──────────┘│
         │         │              │
         ▼         │              │
    ┌──────────┐   │              │
    │  Global  │   │              │
    │ category │   │              │
    └────┬─────┘   │              │
         │         │              │
         ▼         │              │
    ┌──────────┐   │              │
    │ Project  │   │              │
    │   base   │   │              │
    └────┬─────┘   │              │
         │         │              │
         ▼         │              │
    ┌──────────┐   │              │
    │  Global  │   │              │
    │   base   │   │              │
    └────┬─────┘   │              │
         │         │              │
         ▼         │              │
    ┌──────────┐   │              │
    │  Denied  │   │              │
    │(default) │   │              │
    └──────────┘   │              │
```

## Configuration

### Basic Structure

```toml
[access_control]
# Global base level
all = "read"

# Global patterns
deny = ["delete_.*"]
allow = ["delete_issue_note"]

# Category settings
[access_control.categories.issues]
level = "full"
deny = []
allow = []

# Individual action overrides
[access_control.actions]
merge_merge_request = "deny"
create_pipeline = "allow"

# Project-specific overrides
[access_control.projects."group/project"]
all = "none"
deny = [".*"]
allow = ["list_.*", "get_.*"]

[access_control.projects."group/project".categories.wiki]
level = "full"

[access_control.projects."group/project".actions]
create_issue = "allow"
```

## Common Scenarios

### Read-Only Access

Restrict all operations to read-only:

```toml
[access_control]
all = "read"
```

### Full Access with Protected Operations

Allow most operations but prevent destructive ones:

```toml
[access_control]
all = "full"
deny = ["delete_.*", "merge_merge_request"]
```

### Category-Based Access

Enable write access for specific categories:

```toml
[access_control]
all = "read"

[access_control.categories.issues]
level = "full"

[access_control.categories.merge_requests]
level = "full"
deny = ["merge_merge_request"]
```

### Production vs Development

Different access for different projects:

```toml
[access_control]
all = "read"

# Production: strictly read-only
[access_control.projects."company/production"]
all = "read"
deny = [".*"]
allow = ["list_.*", "get_.*", "search_.*"]

# Development: full access
[access_control.projects."company/dev"]
all = "full"
```

### Documentation Project

Allow wiki edits only:

```toml
[access_control]
all = "none"

[access_control.projects."company/docs"]
all = "read"

[access_control.projects."company/docs".categories.wiki]
level = "full"
```

### CI/CD Management

Allow pipeline operations but restrict repository changes:

```toml
[access_control]
all = "read"

[access_control.categories.pipelines]
level = "full"

[access_control.categories.repository]
level = "read"

[access_control.categories.branches]
level = "read"
```

## Pattern Matching

Patterns use regex syntax and match against tool names:

| Pattern | Matches |
|---------|---------|
| `delete_.*` | All delete operations |
| `.*_issue` | Operations ending with "_issue" |
| `list_.*` | All list operations |
| `merge_merge_request` | Exact match |
| `.*pipeline.*` | Any tool containing "pipeline" |

### Pattern Priority

Within a single level:
1. `deny` patterns are checked first
2. If denied, check `allow` patterns
3. `allow` can override `deny` at the same level

```toml
[access_control]
deny = ["delete_.*"]           # Deny all deletes
allow = ["delete_issue_note"]  # But allow deleting issue notes
```

## Environment Variables

All access control settings can be overridden via environment variables:

```bash
# Base level
TANUKI_MCP_ACCESS_CONTROL__ALL=read

# Note: Complex nested structures are best configured via file
```

## Security Recommendations

1. **Start restrictive**: Begin with `all = "read"` and enable features as needed
2. **Protect production**: Use project-specific overrides for production environments
3. **Limit destructive operations**: Consider denying all `delete_.*` patterns globally
4. **Review regularly**: Audit access patterns through the dashboard
5. **Use patterns wisely**: Prefer specific patterns over broad wildcards
