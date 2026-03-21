# --- STAGE 1: Builder ---
FROM rust:1.93-slim-bookworm AS builder

# [FIX]: Sistem bağımlılıkları.
# libasound2-dev: cpal kütüphanesi derlenirken linkleme için gereklidir (kullanılmasa bile).
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    clang \
    protobuf-compiler \
    pkg-config \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/sentiric
COPY . .

WORKDIR /usr/src/sentiric/sentiric-sip-cli

# Build (Release)
RUN cargo build --release

# --- STAGE 2: Runtime ---
FROM debian:bookworm-slim

# [FIX]: Runtime kütüphaneleri.
# libasound2: Binary çalışırken dinamik linkleme hatası almamak için gereklidir.
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/sentiric/sentiric-sip-cli/target/release/sentiric-sip-cli /usr/local/bin/sentiric-sip-cli

# Varsayılan olarak help basar, argümanları kullanıcıdan bekler
ENTRYPOINT ["sentiric-sip-cli"]