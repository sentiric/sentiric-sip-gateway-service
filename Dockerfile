# --- STAGE 1: Builder (Statik Derleme için MUSL Hedefi) ---
FROM rust:1.88-bullseye AS builder

# MUSL target'ını ekliyoruz ve gerekli derleme araçlarını kuruyoruz.
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools protobuf-compiler git curl

WORKDIR /app

# Bağımlılıkları önbelleğe al
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
# MUSL hedefiyle derliyoruz
RUN cargo build --release --target x86_64-unknown-linux-musl

# Kaynak kodunu kopyala ve asıl derlemeyi yap
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# --- STAGE 2: Final (Minimal ve İşlevsel İmaj) ---
# DÜZELTME: scratch yerine alpine kullanıyoruz.
FROM alpine:latest

# Alpine, temel sistem dosyalarını ve kütüphaneleri içerir.
# Ekstra olarak, TLS doğrulaması için ca-certificates ekliyoruz.
RUN apk add --no-cache ca-certificates

ARG SERVICE_NAME
WORKDIR /app

# Statik olarak derlenmiş binary'yi kopyala
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/${SERVICE_NAME} .

# Güvenlik için, programı root olmayan bir kullanıcı olarak çalıştır.
# Alpine içinde 'nobody' kullanıcısı varsayılan olarak gelir (ID 65534).
USER nobody

# ENTRYPOINT ["./sentiric-sip-gateway-service"]
CMD ["tail", "-f", "/dev/null"]