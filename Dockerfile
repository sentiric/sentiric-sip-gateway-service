# --- AŞAMA 1: Derleme (Builder) ---
FROM rust:1.82 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler clang libclang-dev
RUN rustup toolchain install nightly && rustup default nightly

WORKDIR /app

# Bağımlılıkları önbelleğe almak için önce sadece Cargo dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./
# Sahte bir src dizini oluşturarak sadece bağımlılıkları derle
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/sentiric_sip_gateway_service*

# Kaynak kodunu kopyala ve asıl derlemeyi yap
COPY src ./src
# Sadece değişen kodun tekrar derlenmesini sağlar
RUN cargo build --release

# --- AŞAMA 2: Çalıştırma (Runtime) ---
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/sentiric-sip-gateway-service .
EXPOSE 5060/udp
ENTRYPOINT ["/app/sentiric-sip-gateway-service"]