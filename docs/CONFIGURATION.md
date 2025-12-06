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

All configuration options can be set via environment variables with the `TANUKI_MCP_` prefix.

Nested configuration uses double underscores (`__`) as separators:

```bash
# GitLab settings
TANUKI_MCP_GITLAB__URL=https://gitlab.example.com
TANUKI_MCP_GITLAB__TOKEN=glpat-xxxxxxxxxxxx
TANUKI_MCP_GITLAB__TIMEOUT_SECS=60

# Server settings
TANUKI_MCP_SERVER__TRANSPORT=http
TANUKI_MCP_SERVER__HOST=0.0.0.0
TANUKI_MCP_SERVER__PORT=20289

# Dashboard settings
TANUKI_MCP_DASHBOARD__ENABLED=true
TANUKI_MCP_DASHBOARD__HOST=127.0.0.1
TANUKI_MCP_DASHBOARD__PORT=19892

# Access control base level
TANUKI_MCP_ACCESS_CONTROL__ALL=read
```

## Command-Line Arguments

```bash
tanuki-mcp [OPTIONS]

Options:
    --config <PATH>       Path to configuration file
    --http                Use HTTP/SSE transport instead of stdio
    --host <HOST>         HTTP server host [default: 127.0.0.1]
    --port <PORT>         HTTP server port [default: 20289]
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
# - http: HTTP with Server-Sent Events (for web clients)
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
# Recommended: Set via TANUKI_MCP_GITLAB__TOKEN environment variable
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

# Request timeout in seconds
timeout_secs = 30

# Maximum retries for failed requests
max_retries = 3

# Verify SSL certificates
verify_ssl = true

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
# Access Control
# =============================================================================
[access_control]
# Base access level: "none", "read", or "full"
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
        "TANUKI_MCP_GITLAB__URL": "https://gitlab.com",
        "TANUKI_MCP_GITLAB__TOKEN": "glpat-xxx"
      }
    }
  }
}
```

### HTTP/SSE

HTTP transport with Server-Sent Events for web clients.

```bash
# Run with HTTP transport
tanuki-mcp --http

# With custom host/port
tanuki-mcp --http --host 0.0.0.0 --port 8080
```

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
