# Default recipe - list available recipes
default:
    @just --list

# Run all checks (same as CI)
check: fmt-check clippy test doc

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features

# Run unit tests (excludes e2e)
test:
    cargo test

# Build documentation
doc:
    RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --all-features

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Install pre-commit hooks
pre-commit-install:
    pre-commit install

# Run pre-commit on all files
pre-commit-run:
    pre-commit run --all-files

# --- E2E Tests ---

# Run E2E tests with both transports (requires Docker)
e2e: build _e2e-gitlab-up _e2e-wait _e2e-runner-setup && _e2e-gitlab-down
    #!/usr/bin/env bash
    set -euo pipefail
    just _e2e-run

# Run E2E tests with stdio transport only (no HTTP server)
e2e-stdio: build _e2e-gitlab-up _e2e-wait _e2e-runner-setup && _e2e-gitlab-down
    #!/usr/bin/env bash
    set -euo pipefail
    just _e2e-run-stdio-only

# Start GitLab CE container for E2E tests
e2e-gitlab-up:
    docker compose -f e2e/docker-compose.yml --profile infra up -d

# Stop GitLab CE container
e2e-gitlab-down:
    docker compose -f e2e/docker-compose.yml --profile infra --profile mcp down -v

# Show GitLab container logs
e2e-gitlab-logs:
    docker compose -f e2e/docker-compose.yml logs -f gitlab

# Show MCP server container logs
e2e-mcp-logs:
    docker compose -f e2e/docker-compose.yml --profile mcp logs -f

# Check GitLab container status
e2e-status:
    docker compose -f e2e/docker-compose.yml ps
    @curl -sf http://localhost:8080/users/sign_in >/dev/null 2>&1 && echo "GitLab is healthy and ready" || echo "GitLab is not ready yet"

# Internal: Start GitLab (idempotent)
[private]
_e2e-gitlab-up:
    docker compose -f e2e/docker-compose.yml --profile infra up -d

# Internal: Stop GitLab
[private]
_e2e-gitlab-down:
    docker compose -f e2e/docker-compose.yml --profile infra --profile mcp down -v

# Internal: Wait for GitLab to be ready
[private]
_e2e-wait:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Waiting for GitLab to be ready (this may take 3-5 minutes)..."
    until curl -sf http://localhost:8080/users/sign_in >/dev/null 2>&1; do
      echo "  Waiting for login page..."
      sleep 10
    done
    echo "GitLab is ready!"

# Internal: Setup GitLab runner
[private]
_e2e-runner-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Setting up GitLab runner..."
    TOKEN=$(docker exec tanuki-mcp-e2e-gitlab gitlab-rails runner "
      Ci::Runner.where(description: 'e2e-runner').destroy_all
      runner = Ci::Runner.create!(
        runner_type: :instance_type,
        description: 'e2e-runner',
        run_untagged: true,
        active: true
      )
      puts runner.token
    " 2>/dev/null | tail -1)
    if [ -z "$TOKEN" ]; then
      echo "ERROR: Failed to create runner token"
      exit 1
    fi
    echo "Created runner token"
    docker exec tanuki-mcp-e2e-gitlab-runner gitlab-runner unregister --all-runners 2>/dev/null || true
    echo "Registering runner..."
    docker exec tanuki-mcp-e2e-gitlab-runner gitlab-runner register \
      --non-interactive \
      --url http://gitlab:80 \
      --token "$TOKEN" \
      --executor shell \
      --description "e2e-shell-runner"
    echo "Starting runner daemon..."
    docker exec -d tanuki-mcp-e2e-gitlab-runner gitlab-runner run
    sleep 2
    echo "GitLab runner setup complete!"

# Internal: Run E2E tests with MCP HTTP server
[private]
_e2e-run:
    #!/usr/bin/env bash
    set -euo pipefail
    TOKEN=$(./e2e/scripts/create-pat.sh)
    export GITLAB_URL=http://localhost:8080
    export GITLAB_TOKEN="$TOKEN"
    export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"

    echo "Starting MCP HTTP server container..."
    docker compose -f e2e/docker-compose.yml --profile mcp up -d --build

    cleanup() {
      echo "Stopping MCP server container..."
      docker compose -f e2e/docker-compose.yml --profile mcp stop
    }
    trap cleanup EXIT

    echo "Waiting for MCP server health endpoint..."
    for i in $(seq 1 30); do
      curl -sf http://localhost:20399/health >/dev/null 2>&1 && break
      sleep 1
    done
    curl -sf http://localhost:20399/health >/dev/null 2>&1 || { echo "ERROR: MCP server health check failed after 30s"; docker compose -f e2e/docker-compose.yml --profile mcp logs; exit 1; }
    echo "MCP server is ready"

    export MCP_HTTP_URL=http://localhost:20399/mcp
    cargo test -p tanuki-mcp-e2e

# Internal: Run E2E tests stdio only
[private]
_e2e-run-stdio-only:
    #!/usr/bin/env bash
    set -euo pipefail
    TOKEN=$(./e2e/scripts/create-pat.sh)
    export GITLAB_URL=http://localhost:8080
    export GITLAB_TOKEN="$TOKEN"
    export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
    echo "Running stdio-only tests (HTTP tests will be filtered out)..."
    cargo test -p tanuki-mcp-e2e -- --skip "case_2_http"

# --- Development Environment ---

# Start full development environment (GitLab + MCP + Inspector)
dev: _e2e-gitlab-up _e2e-wait _e2e-runner-setup _dev-mcp-up
    @echo ""
    @echo "=== Development Environment Ready ==="
    @echo "GitLab:        http://localhost:8080 (root/testpassword123!)"
    @echo "MCP Server:    http://localhost:20399/mcp"
    @echo "Dashboard:     http://localhost:20400"
    @echo "MCP Inspector: http://localhost:6274"
    @echo ""
    @echo "To connect Inspector to MCP server:"
    @echo "  Transport: Streamable HTTP"
    @echo "  URL: http://tanuki-mcp:20289/mcp"
    @echo ""
    @echo "Stop with: just dev-down"

# Internal: Start MCP server and Inspector
[private]
_dev-mcp-up:
    #!/usr/bin/env bash
    set -euo pipefail
    TOKEN=$(./e2e/scripts/create-pat.sh)
    export GITLAB_TOKEN="$TOKEN"
    docker compose -f e2e/docker-compose.yml --profile mcp --profile inspector up -d --build

    echo "Waiting for MCP server..."
    for i in $(seq 1 30); do
      curl -sf http://localhost:20399/health >/dev/null 2>&1 && break
      sleep 1
    done
    curl -sf http://localhost:20399/health >/dev/null 2>&1 || echo "WARNING: MCP server not responding"

# Stop development environment
dev-down:
    docker compose -f e2e/docker-compose.yml --profile infra --profile mcp --profile inspector down -v

# --- Release Management ---

# Create a release: just release patch|minor|major|x.y.z
release version:
    #!/usr/bin/env bash
    set -euo pipefail

    git diff --quiet || (echo "ERROR: Working directory not clean" && exit 1)

    echo "Bumping version..."
    case "{{version}}" in
      patch|minor|major) cargo set-version --workspace --bump "{{version}}" ;;
      *) cargo set-version --workspace "{{version}}" ;;
    esac
    cargo generate-lockfile

    RELEASE_VERSION=$(cargo pkgid | cut -d "@" -f2)
    echo "=== Release v$RELEASE_VERSION ==="

    git add Cargo.toml tanuki-mcp-macros/Cargo.toml e2e/Cargo.toml Cargo.lock
    git commit -m "chore: release v$RELEASE_VERSION"

    echo "Running checks..."
    just check

    git tag -a "v$RELEASE_VERSION" -m "Release v$RELEASE_VERSION"
    echo "Created tag v$RELEASE_VERSION"
    echo ""
    echo "=== Release v$RELEASE_VERSION created successfully ==="
    echo "To complete the release, push the tag and commit:"
    echo "  git push origin main"
    echo "  git push origin v$RELEASE_VERSION"

# Create a release, skipping E2E tests
release-skip-e2e version:
    #!/usr/bin/env bash
    set -euo pipefail
    just release {{version}}
