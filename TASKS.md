# 🛡️ Sentiric SIP Gateway Service - Görev Listesi (v2.0 - Modüler & Sağlam)

Bu belge, `sip-gateway-service`'in geliştirme yol haritasını, tamamlanan önemli kilometre taşlarını ve gelecekteki hedeflerini tanımlar.

---

### **FAZ 1: Temel Proxy ve NAT Çözümü (Tamamlanmış Görevler)**
Bu faz, servisin temel işlevselliğini sağlayan ilk adımları içerir.

*   [x] **GW-CORE-01: UDP Sunucusu ve Paket Yönlendirme:** Gelen ham UDP paketlerini dinleme ve `sip-signaling-service`'e yönlendirme.
*   [x] **GW-CORE-02: Temel İşlem Takibi:** `Call-ID` ve `CSeq` kullanarak istek ve yanıtları eşleştirme ve yanıtları doğru istemciye geri gönderme.
*   [x] **GW-CORE-03: Eski İşlemleri Temizleme:** Belirli bir süre yanıt almayan işlemleri hafızadan temizleyen bir arka plan görevi.

---

### **FAZ 2: Güvenilir Çağrı Kontrolü ve SBC Yetenekleri (Mevcut Durum - Tamamlandı)**
Bu faz, servisi basit bir proxy'den, NAT arkasındaki karmaşık çağrı senaryolarını çözebilen ve daha güvenilir hale gelen bir Session Border Controller (SBC) yeteneklerine kavuşturmayı hedefler.

-   **Görev ID: GW-REFACTOR-01 - Anayasa Uyumlu Modüler Mimariye Geçiş**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Problem Tanımı:** Servisin tüm mantığı tek bir `main.rs` dosyasında toplanmıştı. Bu, kodun okunmasını, bakımını ve test edilmesini zorlaştırıyordu. Ayrıca, loglama standart dışı ve gürültülüydü.
    -   **Çözüm Stratejisi:** Kod tabanı, `config`, `error`, `network` ve `sip` (içerisinde `handler`, `processor`, `transaction`) gibi ayrı modüllere bölündü. `main.rs` sadece servisleri başlatan bir giriş noktası haline getirildi. Loglama, `INFO` seviyesini sadece kritik olaylar için kullanacak, teknik detayları ise `DEBUG` seviyesine taşıyacak şekilde yeniden düzenlendi.
    -   **Kabul Kriterleri:**
        -   [x] Kod, anayasadaki katmanlı mimari prensiplerine uygun olarak modüllere ayrılmıştır.
        -   [x] Derleyici, `dead_code` veya `unused_variable` gibi uyarılar vermemektedir.
        -   [x] `INFO` seviyesindeki loglar, bir çağrı akışını net bir şekilde takip etmeye yetecek kadar temiz ve anlamlıdır.

-   **Görev ID: GW-BUG-01 - NAT (Ağ Adresi Çevrimi) Probleminin Çözülmesi**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Problem Tanımı:** Servis, iç ağdan (Docker) dış ağa (Telekom Operatörü) SIP paketleri gönderirken `Via` ve `Contact` gibi kritik başlıkları kendi **genel (public) IP adresiyle** yeniden yazmıyordu. Bu durum, operatörün `ACK` ve diğer yanıtları doğru adrese gönderememesine ve çağrıların "ulaşılamıyor" hatasıyla başarısız olmasına neden oluyordu.
    -   **Çözüm Stratejisi:** `sip/processor.rs` modülü oluşturuldu. Bu modül artık:
        1. Gelen `INVITE` isteklerindeki `Via` başlığını kendi genel IP'siyle güncelleyerek `sip-signaling`'e iletir.
        2. `sip-signaling`'den gelen `200 OK` gibi yanıtlardaki `Via` ve `Contact` başlıklarını, telekom operatörünün anlayacağı şekilde orijinal istemci bilgileri ve kendi genel IP'siyle yeniden yazarak iletir.
    -   **Kabul Kriterleri:**
        -   [x] Bir çağrı yapıldığında, arayan kişi "ulaşılamıyor" anonsu yerine, çağrının çaldığını duymalı veya doğrudan IVR'a bağlanmalıdır.
        -   [x] `BYE` isteği operatöre ulaştığında, operatörden artık `475 Bad URI` hatası alınmamalıdır.

---

### **FAZ 3: Güvenlik ve Dayanıklılık (Sıradaki Öncelik)**
Bu faz, servisi platformu siber saldırılara karşı koruyan bir güvenlik kalkanına dönüştürmeyi hedefler.

-   [ ] **Görev ID: GW-SEC-001 - Hız Sınırlama (Rate Limiting)**
    -   **Açıklama:** Belirli bir IP adresinden saniyede gelebilecek istek sayısını sınırlayan bir "token bucket" veya benzeri bir algoritma implemente et. Bu, basit DoS (Denial-of-Service) saldırılarını önleyecektir.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: GW-SEC-002 - IP Beyaz/Kara Liste**
    -   **Açıklama:** Sadece belirli IP adreslerinden veya IP aralıklarından gelen isteklere izin veren (veya bilinen kötü niyetli IP'leri engelleyen) bir mekanizma ekle.
    -   **Durum:** ⬜ Planlandı.

---

### **FAZ 4: Gelişmiş Protokol Desteği (Gelecek Vizyonu)**
Bu faz, platformun tarayıcı gibi daha modern iletişim kanallarını desteklemesini sağlamayı hedefler.

-   [ ] **Görev ID: GW-PROTO-001 - WebRTC Entegrasyonu (SIP over WebSocket)**
    -   **Açıklama:** Tarayıcı tabanlı istemcilerin (`web-agent-ui`) platformla sesli iletişim kurabilmesi için SIP over WebSocket (WSS) desteği ekle. Bu, gelen WSS trafiğini iç ağdaki standart SIP/UDP'ye çevirmeyi içerir.
    -   **Durum:** ⬜ Planlandı.