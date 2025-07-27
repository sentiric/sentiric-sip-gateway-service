# --- AŞAMA 1: Derleme (Builder) ---
# En güncel ve 'slim' bir temel imaj kullanarak başlıyoruz.
# Bu, hem en son derleyici özelliklerini almamızı sağlar hem de imaj boyutunu küçültür.
FROM rust:1.88-slim-bookworm AS builder

# Gerekli derleme araçlarını kuruyoruz.
RUN apt-get update && apt-get install -y protobuf-compiler clang libclang-dev pkg-config

WORKDIR /app

# Bağımlılıkları önbelleğe almak için önce sadece Cargo dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./
# Sahte bir src dizini oluşturarak sadece bağımlılıkları derle
RUN mkdir src && echo "fn main() {}" > src/main.rs
# --release olmadan derlemek, sadece bağımlılıkları çekmeyi hızlandırır.
RUN cargo build

# Kaynak kodunu kopyala ve asıl derlemeyi yap
COPY src ./src
# Artık production için optimize edilmiş tam bir build yapıyoruz.
RUN cargo build --release

# --- AŞAMA 2: Çalıştırma (Runtime) ---
# Mümkün olan en küçük ve en güvenli imajlardan birini kullanıyoruz.
FROM gcr.io/distroless/cc-debian12

# Derlenmiş uygulamayı builder aşamasından kopyala
# WORKDIR'ı ENTRYPOINT'ten önce tanımlamak iyi bir pratiktir.
WORKDIR /app
# NOT: Buradaki 'sentiric-service-name' kısmını her servisin kendi adıyla değiştirmeliyiz.
COPY --from=builder /app/target/release/sentiric-sip-gateway-service .

# Servisin dış dünyaya açtığı portları belirtmek iyi bir dokümantasyondur.
# EXPOSE 5060/udp 

# Uygulamayı çalıştır
ENTRYPOINT ["./sentiric-sip-gateway-service"]