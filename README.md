# 🛡️ Sentiric SIP Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![Protocol](https://img.shields.io/badge/protocol-SIP_(UDP)-green.svg)]()

**Sentiric SIP Gateway Service**, Sentiric platformunun **zırhlı ön kapısı ve akıllı yönlendiricisidir**. Dış dünyadan gelen "ham" ve potansiyel olarak tehlikeli SIP trafiğini ilk karşılayan, süzen ve platformun içindeki `sentiric-sip-signaling-service`'e temiz bir şekilde ileten kritik bir bileşendir.

Basit bir proxy'den çok daha fazlasıdır; bir **Session Border Controller (SBC)**'nin temel görevlerini üstlenir.

## 🎯 Temel Sorumluluklar

1.  **Güvenlik Kalkanı:**
    *   Platformun geri kalanını internetin tehlikelerinden korur.
    *   **DDoS Koruması (Gelecek):** Anormal istekleri sınırlayarak (rate limiting) platformun çökmesini engeller.
    *   **Protokol Temizliği:** Standartlara uymayan SIP paketlerini reddederek iç sistemlerin kararlılığını korur.
2.  **Ağ Tercümanı (NAT Traversal):**
    *   Yerel ağlar (LAN) arkasındaki SIP istemcilerinin (örn: MicroSIP) platformla sorunsuz sesli iletişim kurmasını sağlar.
    *   `Via` ve `Contact` gibi SIP başlıklarını akıllıca yöneterek, gelen paketlerin nereden geldiğini ve yanıtların nereye gitmesi gerektiğini doğru bir şekilde belirler.
3.  **Yönlendirme (Routing):**
    *   Gelen tüm SIP isteklerini, platformun sinyal işleme mantığını barındıran tek bir hedefe (`sentiric-sip-signaling-service`) yönlendirir.

## 🛠️ Teknoloji Yığını

*   **Dil:** Rust
*   **Asenkron Runtime:** Tokio
*   **Gözlemlenebilirlik:** `tracing` ile yapılandırılmış, ortama duyarlı loglama.

## 🔌 API Etkileşimleri

*   **Gelen (Protokol):**
    *   Harici Telekom Sağlayıcıları / SIP İstemcileri (SIP/UDP)
*   **Giden (Protokol):**
    *   `sentiric-sip-signaling-service` (SIP/UDP): Temizlenmiş ve yönlendirilmiş SIP trafiğini iletir.

## 🚀 Yerel Geliştirme

1.  **Bağımlılıkları Yükleyin:** `cargo build`
2.  **`.env` Dosyasını Oluşturun:** `SIP_GATEWAY_LISTEN_PORT` ve hedef `SIP_SIGNALING_SERVICE_HOST`/`PORT`'unu tanımlayın.
3.  **Servisi Çalıştırın:** `cargo run --release`

## 🤝 Katkıda Bulunma

Katkılarınızı bekliyoruz! Lütfen projenin ana [Sentiric Governance](https://github.com/sentiric/sentiric-governance) reposundaki kodlama standartlarına ve katkıda bulunma rehberine göz atın.

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın (v11.0)](https://github.com/sentiric/sentiric-governance/blob/main/docs/blueprint/Architecture-Overview.md) **Zeka & Orkestrasyon Katmanı**'nda yer alan merkezi bir bileşendir.