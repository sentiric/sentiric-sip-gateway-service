# 🛡️ Sentiric SIP Gateway Service - Görev Listesi

Bu belge, `sip-gateway-service`'in geliştirme yol haritasını ve önceliklerini tanımlar.

---

### Faz 1: Temel Proxy ve NAT Çözümü (Mevcut Durum)

Bu faz, servisin temel yönlendirme ve ağ adresi dönüşümü görevlerini yerine getirmesini hedefler.

-   [x] **UDP Sunucusu:** Belirtilen porttan ham UDP paketlerini dinleme.
-   [x] **Paket Yönlendirme:** Gelen paketleri `sip-signaling-service`'e, giden paketleri ise orijinal istemci adresine yönlendirme.
-   [x] **İşlem Takibi (Transaction Matching):** `Call-ID` kullanarak istek ve yanıtları eşleştirme ve yanıtları doğru istemciye geri gönderme.
-   [x] **Eski İşlemleri Temizleme:** Belirli bir süre yanıt almayan işlemleri hafızadan temizleyen bir arka plan görevi.

-   [ ] **Görev ID:** `SIG-BUG-01`
    *   **Açıklama:** `agent-service`'ten gelen sonlandırma isteği üzerine `sip-signaling` tarafından gönderilen `BYE` paketinin neden istemci tarafından işlenmediğini araştır ve düzelt. Bu, `Via`, `Route`, `Record-Route` başlıklarının doğru yönetilmesini gerektirebilir.
    *   **Kabul Kriterleri:**
        *   [ ] Sistem "Çağrıyı sonlandırıyorum" anonsunu çaldıktan sonra, softphone'un çağrıyı **otomatik olarak kapatması** gerekir.

---

### Faz 2: Güvenlik ve Dayanıklılık (Sıradaki Öncelik)

Bu faz, servisi basit bir proxy'den, platformu koruyan bir güvenlik kalkanına dönüştürmeyi hedefler.

-   [ ] **Görev ID: GW-SIP-001 - Hız Sınırlama (Rate Limiting)**
    -   **Açıklama:** Belirli bir IP adresinden saniyede gelebilecek istek sayısını sınırlayan bir "token bucket" veya benzeri bir algoritma implemente et. Bu, basit DoS saldırılarını önleyecektir.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: GW-SIP-002 - IP Beyaz/Kara Liste**
    -   **Açıklama:** Sadece belirli IP adreslerinden veya IP aralıklarından gelen isteklere izin veren (veya bilinen kötü niyetli IP'leri engelleyen) bir mekanizma ekle.
    -   **Durum:** ⬜ Planlandı.

---

### Faz 3: Gelişmiş Protokol Desteği

Bu faz, platformun daha modern iletişim kanallarını desteklemesini sağlamayı hedefler.

-   [ ] **Görev ID: GW-SIP-003 - WebRTC Entegrasyonu (SIP over WebSocket)**
    -   **Açıklama:** Tarayıcı tabanlı istemcilerin (`web-agent-ui`) platformla sesli iletişim kurabilmesi için SIP over WebSocket (WSS) desteği ekle. Bu, gelen WSS trafiğini iç ağdaki standart SIP/UDP'ye çevirmeyi içerir.
    -   **Durum:** ⬜ Planlandı.