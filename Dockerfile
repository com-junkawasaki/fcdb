# FCDB (Enishi) Dockerfile
# Multi-stage build for optimal image size and security

# Build stage
FROM rust:1.70-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
ENV USER=fcdb
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /fcdb

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (for caching)
RUN cargo build --release && rm -rf src/

# Copy source code
COPY src/ src/
COPY fcdb-core/ fcdb-core/
COPY fcdb-cas/ fcdb-cas/
COPY fcdb-graph/ fcdb-graph/
COPY fcdb-exec/ fcdb-exec/
COPY fcdb-concur/ fcdb-concur/
COPY fcdb-api/ fcdb-api/
COPY fcdb-tools/ fcdb-tools/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Import user from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy binary from builder
COPY --from=builder /fcdb/target/release/fcdb /usr/local/bin/fcdb

# Use non-root user
USER fcdb:fcdb

# Expose ports (adjust based on your API configuration)
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["fcdb", "--config", "/etc/fcdb/config.toml"]
