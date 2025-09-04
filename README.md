# 🛡️ Sentiric SIP Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![Protocol](https://img.shields.io/badge/protocol-SIP_(UDP)-green.svg)]()

**Sentiric SIP Gateway Service**, Sentiric platformunun **zırhlı ön kapısı ve akıllı ağ tercümanıdır**. Dış dünyadan (Telekom Operatörleri, SIP İstemcileri) gelen "ham" ve potansiyel olarak güvensiz SIP trafiğini ilk karşılayan, süzen, temizleyen ve platformun içindeki `sentiric-sip-signaling-service`'e güvenli bir şekilde ileten kritik bir bileşendir.

Bu servis, basit bir proxy'den çok daha fazlasıdır; bir **Oturum Sınır Denetleyicisi'nin (Session Border Controller - SBC)** temel görevlerini üstlenir.

## 🎯 Temel Sorumluluklar

1.  **Ağ Sınırı ve Güvenlik Kalkanı (Topology Hiding):**
    *   Platformun iç ağ yapısını (kullanılan servisler, özel IP adresleri) dış dünyadan tamamen gizler. Dışarıdan bakıldığında, tüm platform tek bir IP adresi gibi görünür.
    *   Protokol Temizliği: Standartlara uymayan veya bozuk SIP paketlerini reddederek iç sistemlerin kararlılığını korur.
    *   **DDoS Koruması (Gelecek):** Anormal istekleri sınırlayarak (rate limiting) platformun çökmesini engeller.

2.  **Ağ Adresi Çevrimi (NAT Traversal):**
    *   Platformun Docker gibi özel ağlar içinde çalışmasını mümkün kılar.
    *   `Via` ve `Contact` gibi kritik SIP başlıklarını, paketin yönüne (içeriden dışarıya / dışarıdan içeriye) göre akıllıca yeniden yazarak, NAT arkasındaki servislerin dış dünya ile sorunsuz iletişim kurmasını sağlar. **Bu, servisin en kritik görevidir.**

3.  **Tek Noktaya Yönlendirme (Routing):**
    *   Gelen tüm geçerli SIP isteklerini, platformun sinyal işleme mantığını barındıran tek bir hedefe (`sentiric-sip-signaling-service`) yönlendirir.

## 🛠️ Teknoloji Yığını

*   **Dil:** Rust (Yüksek performans, bellek güvenliği ve düşük seviyeli ağ kontrolü için)
*   **Asenkron Runtime:** Tokio
*   **Gözlemlenebilirlik:** `tracing` ile yapılandırılmış, ortama duyarlı (JSON/Console) ve seviyelendirilmiş (INFO/DEBUG) loglama.
*   **Mimari:** Modüler yapı (config, network, sip, error).

## 🔌 API Etkileşimleri

*   **Gelen (Protokol):**
    *   Harici Telekom Sağlayıcıları / SIP İstemcileri (SIP/UDP)
*   **Giden (Protokol):**
    *   `sentiric-sip-signaling-service` (SIP/UDP): Temizlenmiş ve başlıkları yeniden yazılmış SIP trafiğini iletir.

## 🚀 Yerel Geliştirme

1.  **`.env` Dosyasını Oluşturun:**
    *   Projenin kök dizininde `.env` adında bir dosya oluşturun.
    *   `sentiric-config` reposundaki ilgili profilden (`dev.composite.env` gibi) aşağıdaki değişkenleri kopyalayın ve kendi ortamınıza göre düzenleyin:
        ```env
        # .env
        ENV=development
        RUST_LOG=info,sentiric_sip_gateway_service=debug

        # Hedef sinyal servisinin adresi (Docker içindeki adı)
        SIP_SIGNALING_SERVICE_HOST=sip-signaling
        SIP_SIGNALING_SERVICE_PORT=5060

        # Gateway'in dinleyeceği port
        SIP_GATEWAY_LISTEN_PORT=5060
        
        # !! EN KRİTİK AYAR !!
        # Bu, telekom operatörünün göreceği genel IP adresinizdir.
        # Yerel test için bu, makinenizin LAN IP'si veya 127.0.0.1 olabilir.
        # Bulut sunucu için bu, sunucunun PUBLIC IP adresi OLMALIDIR.
        PUBLIC_IP=192.168.1.100 
        ```

2.  **Bağımlılıkları Yükleyin:**
    ```bash
    cargo build
    ```

3.  **Servisi Çalıştırın:**
    ```bash
    cargo run --release
    ```
    *Not: Log seviyesini değiştirmek için `RUST_LOG=debug cargo run --release` komutunu kullanabilirsiniz.*

## 🤝 Katkıda Bulunma

Katkılarınızı bekliyoruz! Lütfen projenin ana [Sentiric Governance](https://github.com/sentiric/sentiric-governance) reposundaki kodlama standartlarına ve katkıda bulunma rehberine göz atın.

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın (v11.0)](https://github.com/sentiric/sentiric-governance/blob/main/docs/blueprint/Architecture-Overview.md) **Telekom & Medya Katmanı**'nda yer alan, platformun dış dünya ile ilk temas noktası olan kritik bir bileşendir.