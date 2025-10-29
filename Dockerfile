# Multi-stage Dockerfile for OmniTAK
# Stage 1: Build the Rust application
FROM rust:1.90-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/omnitak

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Copy source code
COPY src/ ./src/

# Build for release
RUN cargo build --release --bin omnitak

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 omnitak

# Create directories
RUN mkdir -p /app/config /app/certs && \
    chown -R omnitak:omnitak /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/omnitak/target/release/omnitak /app/omnitak

# Change ownership
RUN chown omnitak:omnitak /app/omnitak

# Switch to app user
USER omnitak

# Expose API port
EXPOSE 8443

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/omnitak", "--help"] || exit 1

# Run the binary
ENTRYPOINT ["/app/omnitak"]
CMD ["--config", "/app/config/config.yaml"]
