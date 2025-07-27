# --- AŞAMA 1: Derleme (Builder) ---
FROM rust:1.79-alpine AS builder

WORKDIR /app

# Bağımlılıkları önbelleğe almak için önce sadece Cargo dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./
# Sahte bir src dizini oluşturarak sadece bağımlılıkları derle
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Kaynak kodunu kopyala ve asıl derlemeyi yap
COPY src ./src
# Sadece değişen kodun tekrar derlenmesini sağlar
RUN cargo build --release

# --- AŞAMA 2: Çalıştırma (Runtime) ---
FROM alpine:latest
# Alpine'da dinamik linkleme için gerekli olan kütüphaneler
RUN apk add --no-cache libc6-compat

WORKDIR /app

# Derlenmiş uygulamayı builder aşamasından kopyala
COPY --from=builder /app/target/release/sentiric-sip-gateway-service .

# Uygulamayı çalıştır
ENTRYPOINT ["./sentiric-sip-gateway-service"]