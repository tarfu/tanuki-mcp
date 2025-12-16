# Configuration

tanuki-mcp uses a layered configuration system with multiple sources.

## Configuration Sources

Configuration is loaded in priority order (later sources override earlier):

1. Default values
2. Configuration file
3. Environment variables
4. Command-line arguments

## Configuration File Locations

The server searches for configuration files in the following order:

1. `./tanuki-mcp.toml` (current directory)
2. `./.tanuki-mcp.toml` (hidden file in current directory)
3. `~/.config/tanuki-mcp/config.toml` (user config)
4. `/etc/tanuki-mcp/config.toml` (system config)

## Environment Variables

All configuration options can be set via environment variables with the `TANUKI_MCP__` prefix
(double underscore after TANUKI_MCP).

Nested configuration uses single underscores (`_`) as separators:

```bash
# GitLab settings
TANUKI_MCP__GITLAB_URL=https://gitlab.example.com
TANUKI_MCP__GITLAB_TOKEN=glpat-xxxxxxxxxxxx
TANUKI_MCP__GITLAB_TIMEOUT_SECS=60

# Server settings
TANUKI_MCP__SERVER_TRANSPORT=http
TANUKI_MCP__SERVER_HOST=0.0.0.0
TANUKI_MCP__SERVER_PORT=20289

# Dashboard settings
TANUKI_MCP__DASHBOARD_ENABLED=true
TANUKI_MCP__DASHBOARD_HOST=127.0.0.1
TANUKI_MCP__DASHBOARD_PORT=19892

# Access control base level
TANUKI_MCP__ACCESS_CONTROL_ALL=read
```

### GitLab Environment Variable Fallbacks

For compatibility with existing GitLab tooling, the server also checks standard
GitLab environment variables as fallbacks when `TANUKI_MCP__*` variables are not set.

**Token precedence (highest to lowest):**
1. `TANUKI_MCP__GITLAB_TOKEN`
2. `GITLAB_TOKEN`
3. `GITLAB_PRIVATE_TOKEN`
4. `GITLAB_ACCESS_TOKEN`
5. Config file `gitlab.token`

**URL precedence (highest to lowest):**
1. `TANUKI_MCP__GITLAB_URL`
2. `GITLAB_URL`
3. Config file `gitlab.url`

> **Note:** If you have `GITLAB_TOKEN` set globally for GitLab CLI tools, it will
> be used automatically as a fallback. Set `TANUKI_MCP__GITLAB_TOKEN` if you need a
> different token specifically for this server (it will take precedence).

## Command-Line Arguments

```bash
tanuki-mcp [OPTIONS]

Options:
    --config <PATH>       Path to configuration file
    --http                Use HTTP transport (Streamable HTTP) instead of stdio
    --host <HOST>         HTTP server host [default: 127.0.0.1]
    --port <PORT>         HTTP server port [default: 20289]
    --log-level <LEVEL>   Log level (trace, debug, info, warn, error) [default: info]
    --no-dashboard        Disable the web dashboard
    --dashboard-host      Dashboard host [default: 127.0.0.1]
    --dashboard-port      Dashboard port [default: 19892]
    -h, --help            Print help
    -V, --version         Print version
```

## Complete Configuration Reference

```toml
# =============================================================================
# Server Configuration
# =============================================================================
[server]
# Display name shown to MCP clients
name = "tanuki-mcp"

# Server version
version = "0.1.0"

# Transport mode: "stdio" or "http"
# - stdio: Standard input/output (for Claude Code integration)
# - http: Streamable HTTP (for web clients and programmatic access)
transport = "stdio"

# HTTP server settings (only used when transport = "http")
host = "127.0.0.1"
port = 20289

# =============================================================================
# GitLab Connection
# =============================================================================
[gitlab]
# GitLab instance URL (required)
url = "https://gitlab.com"

# Personal Access Token (required)
# Recommended: Set via TANUKI_MCP__GITLAB_TOKEN environment variable
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

# Request timeout in seconds
timeout_secs = 30

# Maximum retries for failed requests
max_retries = 3

# Verify SSL certificates
verify_ssl = true

# Custom User-Agent header (optional, default: "tanuki-mcp/<version>")
# user_agent = "my-custom-agent/1.0"

# API version (default: "v4", rarely needs to be changed)
# api_version = "v4"

# =============================================================================
# Dashboard Configuration
# =============================================================================
[dashboard]
# Enable the web dashboard
enabled = true

# Dashboard host address
# Use "127.0.0.1" for local access only
# Use "0.0.0.0" to allow external access
host = "127.0.0.1"

# Dashboard port
port = 19892

# =============================================================================
# Logging Configuration
# =============================================================================
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Output format: pretty (human-readable) or json (structured)
format = "pretty"

# =============================================================================
# Access Control
# =============================================================================
#
# > **Note:** The default access level is `full` if not specified. We recommend
# > explicitly setting `all = "read"` and enabling specific categories for
# > production use.

[access_control]
# Base access level: "none", "deny", "read", or "full"
all = "read"

# Global deny patterns (regex)
deny = []

# Global allow patterns (regex, can override deny)
allow = []

# Category-level configuration
[access_control.categories.issues]
level = "full"
deny = []
allow = []

[access_control.categories.merge_requests]
level = "full"
deny = ["merge_merge_request"]

# ... (see ACCESS_CONTROL.md for all categories)

# Individual action overrides
[access_control.actions]
# tool_name = "allow" | "deny"

# Project-specific overrides
[access_control.projects."group/project"]
all = "read"
deny = []
allow = []

[access_control.projects."group/project".categories.issues]
level = "full"

[access_control.projects."group/project".actions]
create_issue = "allow"
```

## GitLab Token Permissions

The Personal Access Token requires appropriate scopes based on the tools you want to use:

| Scope | Required For |
|-------|--------------|
| `read_api` | All read operations |
| `api` | All write/delete operations |
| `read_repository` | Repository file access |
| `write_repository` | Repository modifications |

**Recommended**: Use `api` scope for full functionality, or `read_api` for read-only mode.

## Transport Modes

### stdio (Default)

Standard input/output transport for integration with Claude Code and similar tools.

```bash
# Run with stdio transport
tanuki-mcp

# Or explicitly
tanuki-mcp --transport stdio
```

Configure in Claude Code's MCP settings:

```json
{
  "mcpServers": {
    "tanuki-mcp": {
      "command": "tanuki-mcp",
      "env": {
        "TANUKI_MCP__GITLAB_URL": "https://gitlab.com",
        "TANUKI_MCP__GITLAB_TOKEN": "glpat-xxx"
      }
    }
  }
}
```

### HTTP (Streamable HTTP)

HTTP transport using Streamable HTTP protocol for web clients and programmatic access.

```bash
# Run with HTTP transport
tanuki-mcp --http

# With custom host/port
tanuki-mcp --http --host 0.0.0.0 --port 8080
```

**Endpoints:**
- `/mcp` - MCP protocol endpoint (Streamable HTTP)
- `/health` - Health check endpoint (`{"status": "ok"}`)

## Dashboard

The dashboard provides a web interface for monitoring:

- Configuration summary
- Active projects being accessed
- Tool usage statistics
- Category breakdown
- Recent request log

Access at `http://localhost:19892` (default).

### Dashboard Options

```bash
# Disable dashboard
tanuki-mcp --no-dashboard

# Custom host/port
tanuki-mcp --dashboard-host 0.0.0.0 --dashboard-port 8888
```

### Port Auto-Discovery

If the configured port is in use, the server will:
1. Try the next 10 consecutive ports
2. Fall back to OS-assigned port

The actual port is logged on startup.

## Prompts

tanuki-mcp includes built-in prompts for common GitLab workflows:

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `analyze_issue` | Analyze an issue with full context (discussions, related MRs) | `project`, `issue_iid` |
| `review_merge_request` | Review an MR with changes and discussions | `project`, `mr_iid` |

Prompts are always available when the underlying tools have access. Their access follows the same rules as the tools they use internally (e.g., `analyze_issue` requires read access to issues and merge requests).

## Resources

tanuki-mcp supports reading GitLab repository files using the `gitlab://` URI scheme:

```
gitlab://{project}/{file_path}?ref={branch}
```

- **project**: URL-encoded project path (e.g., `group%2Fproject`)
- **file_path**: Path to file in repository
- **ref**: Optional git reference (branch, tag, commit) - defaults to HEAD

**Examples:**
- `gitlab://group%2Fproject/README.md` - Read README from default branch
- `gitlab://group%2Fproject/src/main.rs?ref=develop` - Read file from develop branch

## Examples

### Minimal Configuration

```toml
[gitlab]
url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"
```

### Read-Only Mode

```toml
[gitlab]
url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

[access_control]
all = "read"
```

### HTTP Server with External Access

```toml
[server]
transport = "http"
host = "0.0.0.0"
port = 8080

[gitlab]
url = "https://gitlab.example.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

[dashboard]
host = "0.0.0.0"
port = 8081
```

### Production-Safe Configuration

```toml
[gitlab]
url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

[access_control]
all = "read"
deny = ["delete_.*", "merge_merge_request"]

[access_control.categories.issues]
level = "full"

[access_control.categories.merge_requests]
level = "full"
deny = ["merge_merge_request"]

[access_control.projects."company/production"]
all = "read"
deny = [".*"]
allow = ["list_.*", "get_.*"]
```
