# syntax=docker/dockerfile:1

# Stage 1: Planner - analyze dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.85 AS planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY docs/ docs/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - cache dependencies and build
FROM lukemathwalker/cargo-chef:latest-rust-1.85 AS builder
WORKDIR /app

# Copy recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build application with size optimizations
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --bin three-good-sources

# Stage 3: Runtime - minimal production image
FROM debian:bookworm-slim AS runtime

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user (security best practice)
RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m -d /home/appuser appuser

# Copy binary from builder with correct ownership
COPY --from=builder --chown=appuser:appgroup \
    /app/target/release/three-good-sources \
    /usr/local/bin/app

# Copy data files to /data/ (DO App Platform may overlay /app/ at runtime)
COPY --chown=appuser:appgroup registry.json /data/registry.json
COPY --chown=appuser:appgroup audit_log.json /data/audit_log.json
COPY --chown=appuser:appgroup identities.json /data/identities.json
COPY --chown=appuser:appgroup contributions.json /data/contributions.json

# Switch to non-root user
USER appuser

# Document exposed port
EXPOSE 3000

# Default environment variables
ENV REGISTRY_PATH=/data/registry.json
ENV AUDIT_LOG_PATH=/data/audit_log.json
ENV IDENTITIES_PATH=/data/identities.json
ENV CONTRIBUTIONS_PATH=/data/contributions.json

# Run application
ENTRYPOINT ["/usr/local/bin/app"]
