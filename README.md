# 🛡️ Sentiric SIP Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![Protocol](https://img.shields.io/badge/protocol-SIP_(UDP)-green.svg)]()

**Sentiric SIP Gateway Service**, Sentiric platformunun **zırhlı ön kapısı ve akıllı ağ tercümanıdır**. Dış dünyadan (Telekom Operatörleri, SIP İstemcileri) gelen ham ve standartlara uymayan SIP trafiğini ilk karşılayan, RFC 3261 standardına göre temizleyen, normalize eden ve platformun içindeki `sentiric-sip-signaling-service`'e güvenli ve basit bir formatta ileten kritik bir bileşendir.

Bu servis, basit bir proxy'den çok daha fazlasıdır; bir **Oturum Sınır Denetleyicisi'nin (Session Border Controller - SBC)** temel görevlerini üstlenir.

## 🎯 Temel Sorumluluklar

1.  **Ağ Sınırı ve Topoloji Gizleme (Topology Hiding):**
    *   Platformun iç ağ yapısını (kullanılan servisler, özel IP adresleri) dış dünyadan tamamen gizler. Dışarıdan bakıldığında, tüm platform tek bir IP adresi gibi görünür.
    *   **Protokol Temizliği ve Standardizasyon:** Standartlara uymayan (örn: çoklu `Via` başlıkları) veya bozuk SIP paketlerini alır ve iç ağ için standart, temiz ve tekil bir formata dönüştürür.

2.  **RFC 3261 Uyumlu NAT Çözümü (NAT Traversal):**
    *   Platformun Docker gibi özel ağlar içinde çalışmasını mümkün kılar.
    *   `Via`, `Contact` ve `Record-Route` gibi kritik SIP başlıklarını, paketin yönüne (içeriden dışarıya / dışarıdan içeriye) göre akıllıca yeniden yazarak, NAT arkasındaki servislerin dış dünya ile sorunsuz iletişim kurmasını sağlar. **Bu, servisin en kritik görevidir.**

3.  **Tek Noktaya Yönlendirme (Routing):**
    *   Gelen tüm geçerli SIP isteklerini, platformun sinyal işleme mantığını barındıran tek bir hedefe (`sentiric-sip-signaling-service`) yönlendirir.

## 🛠️ Teknoloji Yığını

*   **Dil:** Rust (Yüksek performans, bellek güvenliği ve düşük seviyeli ağ kontrolü için)
*   **Asenkron Runtime:** Tokio
*   **Gözlemlenebilirlik:** `tracing` ile yapılandırılmış ve ortama duyarlı loglama.
*   **Mimari:** Modüler yapı (`config`, `network`, `sip`, `error`).

## 🚀 Yerel Geliştirme

1.  **Bağımlılıkları Yükleyin:**
2.  **Ortam Değişkenlerini Ayarlayın:** `.env.example` dosyasını `.env` olarak kopyalayın ve gerekli değişkenleri doldurun.
3.  **Servisi Çalıştırın:**

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın](https://github.com/sentiric/sentiric-governance) **Telekom & Medya Katmanı**'nda yer alan, platformun dış dünya ile ilk ve tek sinyalleşme temas noktası olan kritik bir bileşendir.