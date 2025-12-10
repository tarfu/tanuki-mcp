# tanuki-mcp

A high-performance GitLab MCP (Model Context Protocol) server written in Rust with fine-grained access control.

Inspired by [zereight/gitlab-mcp](https://github.com/zereight/gitlab-mcp).

## Features

- **121 GitLab Tools** across 20 categories
- **Built-in Prompts** for issue analysis and MR review workflows
- **Resource Access** via `gitlab://` URI scheme for file reading
- **Fine-Grained Access Control** with hierarchical overrides
- **Two Transport Modes**: stdio (Claude Code) and HTTP (Streamable HTTP)
- **Real-Time Dashboard** for monitoring usage
- **Project-Specific Permissions** for granular control
- **Pattern-Based Rules** using regex for allow/deny lists

## MCP Capabilities

tanuki-mcp implements the full MCP specification with tools, prompts, and resources.

### Prompts

Built-in workflow prompts for common GitLab tasks:

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `analyze_issue` | Analyze an issue with discussions and related MRs | `project`, `issue_iid` |
| `review_merge_request` | Review an MR with changes and discussions | `project`, `mr_iid` |

**Usage in Claude Code:**
```
Use the analyze_issue prompt for project "group/repo" issue 42
```

### Resources

Read GitLab repository files using the `gitlab://` URI scheme:

```
gitlab://{project}/{file_path}?ref={branch}
```

**Examples:**
- `gitlab://group%2Fproject/README.md` - Default branch
- `gitlab://group%2Fproject/src/main.rs?ref=develop` - Specific branch

Note: Project path must be URL-encoded (`/` → `%2F`)

## Quick Start

### Using Docker

```bash
# Run with stdio transport
docker run -it --rm \
  -e TANUKI_MCP_GITLAB__URL=https://gitlab.com \
  -e TANUKI_MCP_GITLAB__TOKEN=glpat-xxx \
  tanuki-mcp

# Run with HTTP transport
docker run -d \
  -p 20289:20289 \
  -p 19892:19892 \
  -e TANUKI_MCP_GITLAB__URL=https://gitlab.com \
  -e TANUKI_MCP_GITLAB__TOKEN=glpat-xxx \
  tanuki-mcp --http
```

### Using Pre-Built Binary

```bash
# Download from releases
curl -LO https://github.com/tarfu/tanuki-mcp/releases/latest/download/tanuki-mcp
chmod +x tanuki-mcp

# Set token and run
export TANUKI_MCP_GITLAB__TOKEN=glpat-xxx
./tanuki-mcp
```

### Building from Source

```bash
git clone https://github.com/tarfu/tanuki-mcp
cd tanuki-mcp
cargo build --release
./target/release/tanuki-mcp
```

## Configuration

Create `tanuki-mcp.toml`:

```toml
[gitlab]
url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

[access_control]
all = "read"

[access_control.categories.issues]
level = "full"

[access_control.categories.merge_requests]
level = "full"
deny = ["merge_merge_request"]
```

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for complete reference.

## Access Control

tanuki-mcp provides hierarchical access control:

```
Global Base → Category → Action → Project-Specific
```

### Access Levels

| Level | Description |
|-------|-------------|
| `none` | No access |
| `read` | Read-only (list, get, search) |
| `full` | Full access (create, update, delete, execute) |

### Example: Production-Safe Setup

```toml
[access_control]
all = "read"
deny = ["delete_.*"]

[access_control.categories.issues]
level = "full"

[access_control.projects."company/production"]
all = "read"
deny = [".*"]
allow = ["list_.*", "get_.*"]
```

See [docs/ACCESS_CONTROL.md](docs/ACCESS_CONTROL.md) for detailed documentation.

## Tool Categories

| Category | Tools | Description |
|----------|-------|-------------|
| issues | 8 | Issue management |
| issue_notes | 5 | Issue comments |
| issue_links | 3 | Issue relationships |
| merge_requests | 8 | MR management |
| mr_discussions | 7 | MR threads |
| mr_drafts | 7 | Draft notes |
| repository | 7 | Files and search |
| branches | 2 | Branch operations |
| commits | 3 | Commit operations |
| projects | 6 | Project management |
| namespaces | 3 | Namespaces |
| labels | 5 | Labels |
| wiki | 5 | Wiki pages |
| pipelines | 12 | CI/CD |
| milestones | 9 | Milestones |
| releases | 6 | Releases |
| users | 2 | Users |
| groups | 2 | Groups |
| tags | 9 | Git tags |
| search | 5 | Search |

## Transport Modes

### stdio (Default)

For integration with Claude Code:

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

### HTTP (Streamable HTTP)

For web clients and programmatic access:

```bash
tanuki-mcp --http --host 0.0.0.0 --port 8080
```

**Endpoints:**
- `/mcp` - MCP protocol (Streamable HTTP)
- `/health` - Health check (`{"status": "ok"}`)

## Dashboard

Access the monitoring dashboard at `http://localhost:19892`:

- Configuration summary
- Project access statistics
- Tool usage metrics
- Recent request log

```bash
# Disable dashboard
tanuki-mcp --no-dashboard

# Custom port
tanuki-mcp --dashboard-port 9000
```

## Environment Variables

```bash
TANUKI_MCP_GITLAB__URL=https://gitlab.com
TANUKI_MCP_GITLAB__TOKEN=glpat-xxx
TANUKI_MCP_SERVER__TRANSPORT=http
TANUKI_MCP_ACCESS_CONTROL__ALL=read
TANUKI_MCP_DASHBOARD__ENABLED=true
```

## CLI Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--config`, `-c` | Configuration file path | Auto-detected |
| `--http` | Use HTTP transport instead of stdio | false |
| `--host` | HTTP server bind address | 127.0.0.1 |
| `--port` | HTTP server port | 20289 |
| `--log-level` | Log level (trace, debug, info, warn, error) | info |
| `--no-dashboard` | Disable the monitoring dashboard | false |
| `--dashboard-port` | Dashboard server port | 19892 |

## Requirements

- Rust 1.83+ (for building from source)
- GitLab Personal Access Token with appropriate scopes:
  - `read_api` for read operations
  - `api` for full functionality

## Development

### Dependencies

```bash
# Task runner (https://taskfile.dev)
brew install go-task

# For release management (cargo set-version)
cargo install cargo-edit
```

### Available Tasks

```bash
task --list        # List all tasks
task check         # Run all checks (fmt, clippy, test, doc)
task release       # Create a release (tag + version bump)
task e2e           # Run E2E tests
```

### Creating a Release

```bash
# Tag current version, bump minor (runs check + e2e)
task release

# Skip E2E tests
task release SKIP_E2E=true

# Custom version
task release VERSION=1.0.0 NEXT_VERSION=2.0.0
```

## License

MIT
