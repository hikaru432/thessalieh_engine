# syntax=docker/dockerfile:1

# ---- Builder ---------------------------------------------------------------
FROM rust:1-slim-bookworm AS builder

# `ring` (pulled in via rustls) needs a C toolchain to build.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build dependencies first so they're cached separately from source edits.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/engine.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN touch src/engine.rs && cargo build --release

# ---- Runtime ----------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/thessalieh_engine ./thessalieh_engine

# Render injects PORT at runtime; the app reads it and binds 0.0.0.0:$PORT
# (see src/engine.rs). 8080 is just the local-dev default.
ENV PORT=8080
EXPOSE 8080

CMD ["./thessalieh_engine"]
