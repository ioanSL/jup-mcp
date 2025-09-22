# Multi-stage Docker build for Jupiter AG MCP Server
FROM rustlang/rust:nightly-slim AS builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency caching
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src/

# Copy actual source code
COPY src/ ./src/

# Build the actual application
RUN cargo build --release

# Final runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false -m -d /app jupiter-mcp

# Set the working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/jup-mpc ./jup-mpc

# Verify the binary exists and change ownership to the non-root user
RUN ls -la /app/jup-mpc && chown jupiter-mcp:jupiter-mcp /app/jup-mpc

# Switch to non-root user
USER jupiter-mcp

# Set default environment variables
ENV RUST_LOG=info
ENV SOLANA_NETWORK=devnet

# Health check (optional - checks if binary exists and is executable)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD [ -x "./jup-mpc" ] || exit 1

# Run the binary
ENTRYPOINT ["./jup-mpc"]