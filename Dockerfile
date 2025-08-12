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

# --- STAGE 2: Final (Sıfırdan İmaj - En Minimal ve Güvenli) ---
# Artık debian'a bile ihtiyacımız yok. "scratch" tamamen boş bir imajdır.
FROM scratch

ARG SERVICE_NAME
WORKDIR /app

# Statik olarak derlenmiş binary, başka hiçbir şeye ihtiyaç duymaz.
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/${SERVICE_NAME} .

# Güvenlik için, programı root olmayan bir kullanıcı olarak çalıştır.
USER 10001

ENTRYPOINT ["./sentiric-sip-gateway-service"]