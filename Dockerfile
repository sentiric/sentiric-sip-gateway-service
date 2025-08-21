# --- STAGE 1: Builder ---
FROM rust:1.88-bullseye AS builder

RUN apt-get update

WORKDIR /app

COPY . .

RUN cargo build --release

# --- STAGE 2: Final ---
FROM debian:bookworm-slim

RUN apt-get update

WORKDIR /app

COPY --from=builder /app/target/release/sentiric-sip-gateway-service .

ENTRYPOINT ["./sentiric-sip-gateway-service"]