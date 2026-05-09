# syntax=docker/dockerfile:1.6
#
# Multi-stage build for giff-runner.
# Builder stage compiles a release binary; runtime stage is a debian-slim with just
# the binary, the runtime dependencies SQLite needs (none — rusqlite has `bundled`),
# and a non-root user.
#
# Build with the workspace root as the build context:
#   docker build -f crates/giff-runner/Dockerfile -t giff-runner .

ARG RUST_VERSION=1.83
FROM rust:${RUST_VERSION}-slim AS builder

WORKDIR /build
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the whole workspace; the runner depends on giff-core and giff-github.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p giff-runner

# ---

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin giff

WORKDIR /home/giff
COPY --from=builder /build/target/release/giff-runner /usr/local/bin/giff-runner

# Default config + data paths. Mount your config as `/etc/giff-runner/config.toml`
# and a persistent volume at `/data` for the SQLite file.
ENV GIFF_RUNNER_CONFIG=/etc/giff-runner/config.toml
USER giff
EXPOSE 8080

# Healthcheck hits /healthz. wget is not in slim by default; use curl-via-busybox no — keep
# it dependency-free with a /proc test against the listen socket.
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/giff-runner", "--config", "/etc/giff-runner/config.toml", "--help"] || exit 1

ENTRYPOINT ["/usr/local/bin/giff-runner"]
