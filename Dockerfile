# Use cargo-chef for dependency caching in Rust
FROM lukemathwalker/cargo-chef:latest-rust-1.85-slim-bookworm AS chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer is cached unless Cargo.toml/Cargo.lock changes
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# Final runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
# We use python3-minimal and download yt-dlp binary directly to avoid the slow pip install process
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates \
    python3 \
    curl \
    && curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp \
    && chmod a+rx /usr/local/bin/yt-dlp \
    && apt-get purge -y curl \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy both compiled binaries from builder
COPY --from=builder /app/target/release/yt_down /usr/local/bin/
COPY --from=builder /app/target/release/yt_down_web /usr/local/bin/

# Copy Deno binary (faster than pulling the full deno:debian image)
COPY --from=denoland/deno:bin /deno /usr/bin/deno

# Create secrets directory for runtime-mounted cookies
RUN mkdir -p /secrets

WORKDIR /downloads

# Set environment variable for cookies path
ENV COOKIES_PATH=/secrets/cookies.txt

# Default to web server for deployment; override with "yt_down" for CLI usage
ENTRYPOINT ["yt_down_web"]
