# Build stage
FROM rust:1.91-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY tanuki-mcp-macros ./tanuki-mcp-macros
COPY src ./src
COPY assets ./assets

# Remove e2e from workspace members (it's not needed for the main binary)
RUN sed -i 's/members = \[".", "tanuki-mcp-macros", "e2e"\]/members = [".", "tanuki-mcp-macros"]/' Cargo.toml

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 tanuki

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/tanuki-mcp /usr/local/bin/tanuki-mcp

# Switch to non-root user
USER tanuki

# Default to stdio transport (for Claude Code integration)
# Override with --http for HTTP/SSE transport
ENTRYPOINT ["tanuki-mcp"]

# Expose ports for HTTP transport and dashboard
# HTTP/SSE: 20289
# Dashboard: 19892
EXPOSE 20289 19892
