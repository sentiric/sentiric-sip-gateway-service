# --- STAGE 1: Builder ---
FROM rust:1.88-bullseye AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# --- STAGE 2: Final ---
FROM debian:bookworm-slim
ARG SERVICE_NAME
WORKDIR /app
COPY --from=builder /app/target/release/${SERVICE_NAME} .
ENTRYPOINT ["./sentiric-sip-gateway-service"]