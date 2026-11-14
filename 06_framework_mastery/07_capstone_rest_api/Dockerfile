# syntax=docker/dockerfile:1
#
# A multi-stage image for the capstone API: a big image with the Rust
# toolchain compiles the program, and a small one runs it.
#
# This crate is one member of a Cargo workspace, so build from the WORKSPACE
# ROOT (the folder with the top-level Cargo.toml):
#
#   docker build -f 06_framework_mastery/07_capstone_rest_api/Dockerfile -t capstone-rest-api .
#
# or run `docker compose up --build` in this folder (see docker-compose.yml).
# Needs BuildKit (the default builder in current Docker versions) for the
# cache mounts and for Dockerfile.dockerignore.

# ---- Stage 1: build ----------------------------------------------------------
FROM rust:1-slim-bookworm AS build
WORKDIR /src

# The whole workspace: Cargo reads every member's Cargo.toml to load it.
# Dockerfile.dockerignore (next to this file) keeps target/, .git and any
# .env files out of the build context.
COPY . .

# The workspace's rust-toolchain.toml asks for the latest stable Rust, so
# rustup may download it here. The bundled SQLite needs a C compiler: the
# rust image already has gcc.
#
# Cache mounts keep downloaded crates and compiled dependencies between
# builds, so changing one line of code doesn't recompile the world.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p capstone_rest_api \
 && cp target/release/capstone_rest_api /usr/local/bin/capstone_rest_api

# ---- Stage 2: run ------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Never run a server as root: a user with no password, no home and no shell.
# /data holds the SQLite database (a volume, so it survives new images).
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin app \
 && mkdir /data \
 && chown app /data

COPY --from=build /usr/local/bin/capstone_rest_api /usr/local/bin/capstone_rest_api

USER app
ENV APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    DATABASE_URL="sqlite:/data/app.db?mode=rwc" \
    LOG_FORMAT=json
# JWT_SECRET is deliberately NOT set here: secrets are given at run time
# (docker run -e / --env-file, compose's env_file, or a secrets manager),
# never baked into an image that anyone with the image can read.

VOLUME ["/data"]
EXPOSE 8080

# The binary checks itself: `healthcheck` requests /health and exits 0 or 1.
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["capstone_rest_api", "healthcheck"]

# Exec form: the server is PID 1 and receives SIGTERM from `docker stop`,
# which starts its graceful shutdown.
CMD ["capstone_rest_api"]
