# tanuki-mcp

A high-performance GitLab MCP (Model Context Protocol) server written in Rust with fine-grained access control.

Inspired by [zereight/gitlab-mcp](https://github.com/zereight/gitlab-mcp).

## Features

- **120 GitLab Tools** across 21 categories
- **Fine-Grained Access Control** with hierarchical overrides
- **Two Transport Modes**: stdio (for Claude Code) and HTTP/SSE
- **Real-Time Dashboard** for monitoring usage
- **Project-Specific Permissions** for granular control
- **Pattern-Based Rules** using regex for allow/deny lists

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
curl -LO https://github.com/yourusername/tanuki-mcp/releases/latest/download/tanuki-mcp
chmod +x tanuki-mcp

# Set token and run
export TANUKI_MCP_GITLAB__TOKEN=glpat-xxx
./tanuki-mcp
```

### Building from Source

```bash
git clone https://github.com/yourusername/tanuki-mcp
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
| graphql | 1 | GraphQL |
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

### HTTP/SSE

For web clients:

```bash
tanuki-mcp --http --host 0.0.0.0 --port 8080
```

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

## Requirements

- Rust 1.83+ (for building from source)
- GitLab Personal Access Token with appropriate scopes:
  - `read_api` for read operations
  - `api` for full functionality

## License

MIT
