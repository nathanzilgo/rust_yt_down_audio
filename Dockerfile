FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM denoland/deno:debian AS deno-base

FROM debian:bookworm-slim

COPY --from=deno-base /usr/bin/deno /usr/bin/deno

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    ffmpeg \
    ca-certificates \
    && pip3 install --break-system-packages yt-dlp \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/yt_down /usr/local/bin/

WORKDIR /downloads

ENTRYPOINT ["yt_down"]