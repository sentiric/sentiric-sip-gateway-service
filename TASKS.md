# 🛡️ Sentiric SIP Gateway Service - Görev Listesi

Bu belge, `sip-gateway-service`'in geliştirme yol haritasını ve önceliklerini tanımlar.

---

### **FAZ 1: Temel Proxy ve NAT Çözümü (Tamamlanmış Görevler)**
*   [x] **UDP Sunucusu:** Belirtilen porttan ham UDP paketlerini dinleme.
*   [x] **Paket Yönlendirme:** Gelen paketleri `sip-signaling-service`'e, giden paketleri ise orijinal istemci adresine yönlendirme.
*   [x] **İşlem Takibi (Transaction Matching):** `Call-ID` ve `CSeq` kullanarak istek ve yanıtları eşleştirme ve yanıtları doğru istemciye geri gönderme.
*   [x] **Eski İşlemleri Temizleme:** Belirli bir süre yanıt almayan işlemleri hafızadan temizleyen bir arka plan görevi.

---

### **FAZ 2: Güvenilir Çağrı Kontrolü ve SBC Yetenekleri (Mevcut Odak)**

-   **Görev ID: SIG-BUG-01 - Çağrı Sonlandırma (`BYE`) Akışını Sağlamlaştırma**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Problem Tanımı:** Sistem `BYE` gönderdiğinde, `sip-gateway`'in `Via` başlıklarını doğru yönetmemesi nedeniyle paket telekom operatörüne ulaşmıyor ve çağrı açık kalıyordu.
    -   **Çözüm Stratejisi:** `sip-gateway` artık bir Session Border Controller (SBC) gibi davranarak gelen ve giden paketlerdeki `Via` başlıklarını kendi adresiyle modifiye ediyor. Bu, `BYE` gibi daha sonraki isteklerin ve yanıtların doğru rotayı takip etmesini sağlar.
    -   **Kabul Kriterleri:**
        -   [x] `agent-service`, çağrıyı sonlandırma komutunu verdiğinde, kullanıcının telefon hattı **5 saniye içinde otomatik olarak kapanmalıdır.**
        -   [x] `sip-signaling` loglarında artık tekrarlayan "BYE isteği alınan çağrı aktif çağrılar listesinde bulunamadı" uyarısı görülmemelidir.
    -   **Ek Not:** Bu düzeltme sırasında derleyici hatalarına neden olan `AppConfig` alan adı tutarsızlığı ve tip uyuşmazlığı da giderilmiştir.

---

### **FAZ 3: Güvenlik ve Dayanıklılık (Sıradaki Öncelik)**

Bu faz, servisi basit bir proxy'den, platformu koruyan bir güvenlik kalkanına dönüştürmeyi hedefler.

-   [ ] **Görev ID: GW-SIP-001 - Hız Sınırlama (Rate Limiting)**
    -   **Açıklama:** Belirli bir IP adresinden saniyede gelebilecek istek sayısını sınırlayan bir "token bucket" veya benzeri bir algoritma implemente et. Bu, basit DoS saldırılarını önleyecektir.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: GW-SIP-002 - IP Beyaz/Kara Liste**
    -   **Açıklama:** Sadece belirli IP adreslerinden veya IP aralıklarından gelen isteklere izin veren (veya bilinen kötü niyetli IP'leri engelleyen) bir mekanizma ekle.
    -   **Durum:** ⬜ Planlandı.

---

### **FAZ 4: Gelişmiş Protokol Desteği (Gelecek Vizyonu)**

Bu faz, platformun daha modern iletişim kanallarını desteklemesini sağlamayı hedefler.

-   [ ] **Görev ID: GW-SIP-003 - WebRTC Entegrasyonu (SIP over WebSocket)**
    -   **Açıklama:** Tarayıcı tabanlı istemcilerin (`web-agent-ui`) platformla sesli iletişim kurabilmesi için SIP over WebSocket (WSS) desteği ekle. Bu, gelen WSS trafiğini iç ağdaki standart SIP/UDP'ye çevirmeyi içerir.
    -   **Durum:** ⬜ Planlandı.