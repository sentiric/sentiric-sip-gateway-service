# --- STAGE 1: Builder ---
FROM rust:1.88-bullseye AS builder

# YENİ: Build argümanlarını tanımla
ARG GIT_COMMIT
ARG BUILD_DATE
ARG SERVICE_VERSION

WORKDIR /app

COPY . .

# YENİ: Build-time environment değişkenlerini ayarla
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV SERVICE_VERSION=${SERVICE_VERSION}

RUN cargo build --release

# --- STAGE 2: Final ---
FROM debian:bookworm-slim

RUN apt-get update

WORKDIR /app

# YENİ: Build argümanlarını tekrar tanımla ki runtime'da da kullanılabilsin
ARG GIT_COMMIT
ARG BUILD_DATE
ARG SERVICE_VERSION

# YENİ: Argümanları environment değişkenlerine ata
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV SERVICE_VERSION=${SERVICE_VERSION}

COPY --from=builder /app/target/release/sentiric-sip-gateway-service .

ENTRYPOINT ["./sentiric-sip-gateway-service"]