# 🛡️ Sentiric SIP Gateway Service - Görev Listesi (v2.1 - Production'a Hazır Temel)

Bu belge, `sip-gateway-service`'in geliştirme yol haritasını, tamamlanan önemli kilometre taşlarını ve gelecekteki hedeflerini tanımlar.

---

### **FAZ 1: Temel Proxy ve NAT Çözümü (Arşivlendi)**
Bu faz, servisin en temel işlevselliğini sağlayan ilk adımları içeriyordu ve başarıyla aşıldı.

*   [x] **GW-CORE-01: UDP Sunucusu ve Paket Yönlendirme:** Gelen ham UDP paketlerini dinleme ve `sip-signaling-service`'e yönlendirme.
*   [x] **GW-CORE-02: Temel İşlem Takibi:** `Call-ID` ve `CSeq` kullanarak istek ve yanıtları eşleştirme ve yanıtları doğru istemciye geri gönderme.
*   [x] **GW-CORE-03: Eski İşlemleri Temizleme:** Belirli bir süre yanıt almayan işlemleri hafızadan temizleyen bir arka plan görevi.

---

### **FAZ 2: Sağlam SBC Yetenekleri ve Production'a Hazırlık (Mevcut Durum - TAMAMLANDI)**
Bu faz, servisi basit bir proxy'den, karmaşık çağrı senaryolarını çözebilen, dayanıklı ve profesyonel bir Oturum Sınır Denetleyicisi (SBC) haline getirmeyi hedefliyordu. **Bu faz başarıyla tamamlanmıştır.**

-   **Görev ID: GW-REFACTOR-01 - Anayasa Uyumlu Modüler Mimariye Geçiş**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Kazanım:** Kod tabanı, `config`, `error`, `network`, `sip` (içerisinde `handler`, `processor`, `transaction`, `message_builder`) gibi ayrı modüllere bölündü. Bu, bakım ve test edilebilirliği kökten iyileştirdi. Loglama, `tracing` ile endüstri standartlarına getirildi.

-   **Görev ID: GW-BUG-01 - NAT (Ağ Adresi Çevrimi) Probleminin Çözülmesi**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Kazanım:** `Via` ve `Contact` başlıkları, servisin genel (public) IP adresi kullanılarak akıllıca yeniden yazıldı. Bu, NAT arkasındaki platformun dış dünya ile başarılı bir şekilde iletişim kurmasını sağladı.

-   **Görev ID: GW-BUG-02 - Diyalog İçi Yönlendirme (`Route` Başlığı) Sorununun Çözümü**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** **KRİTİK**
    -   **Kazanım:** Servis artık `INVITE` paketlerindeki `Record-Route` başlığını anlıyor ve saklıyor. Diyalog içindeki sonraki istekleri (`BYE` gibi), bu bilgiye dayanarak bir `Route` başlığı ile doğru bir şekilde yönlendiriyor. Bu, çağrı sonlandırma hatalarını (`475 Bad URI`) çözmüştür.

-   **Görev ID: GW-ROBUST-01 - Dayanıklılık ve Graceful Shutdown**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** Yüksek
    -   **Kazanım:** Servis artık bağımlı olduğu `sip-signaling-service` gibi servisler ayakta olmadığında çökmüyor; hatayı loglayıp çalışmaya devam ediyor. Ayrıca, `Ctrl+C` (SIGINT) gibi kapatma sinyallerini yakalayarak temiz ve kontrollü bir şekilde kapanıyor (graceful shutdown).

-   **Görev ID: GW-PERF-01 - Yinelenen Paket Filtreleme**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Öncelik:** Yüksek
    -   **Kazanım:** Gateway, telekom operatörlerinden gelen yinelenen `INVITE` isteklerini filtreleyerek `sip-signaling-service` üzerindeki gereksiz yükü engeller.

---

### **FAZ 3: Güvenlik ve Gelişmiş Gözlemlenebilirlik (Sıradaki Öncelik)**
Bu faz, servisi siber saldırılara karşı koruyan bir güvenlik kalkanına dönüştürmeyi ve operasyonel takibini kolaylaştırmayı hedefler.

-   [ ] **Görev ID: GW-SEC-001 - Hız Sınırlama (Rate Limiting)**
    -   **Açıklama:** Belirli bir IP adresinden saniyede gelebilecek istek sayısını sınırlayan bir "token bucket" veya benzeri bir algoritma implemente et. Bu, basit DoS (Denial-of-Service) saldırılarını önleyecektir.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: GW-SEC-002 - IP Beyaz/Kara Liste**
    -   **Açıklama:** Sadece belirli IP adreslerinden veya IP aralıklarından gelen isteklere izin veren (veya bilinen kötü niyetli IP'leri engelleyen) bir mekanizma ekle.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: GW-OBSERV-001 - Prometheus Metrikleri**
    -   **Açıklama:** Aktif çağrı sayısı, saniyedeki istek sayısı, hata oranları gibi kritik metrikleri bir `/metrics` endpoint'i üzerinden Prometheus formatında sun.
    -   **Durum:** ⬜ Planlandı.

---

### **FAZ 4: Gelişmiş Protokol Desteği (Gelecek Vizyonu)**
Bu faz, platformun tarayıcı gibi daha modern iletişim kanallarını desteklemesini sağlamayı hedefler.

-   [ ] **Görev ID: GW-PROTO-001 - WebRTC Entegrasyonu (SIP over WebSocket)**
    -   **Açıklama:** Tarayıcı tabanlı istemcilerin (`web-agent-ui`) platformla sesli iletişim kurabilmesi için SIP over WebSocket (WSS) desteği ekle. Bu, gelen WSS trafiğini iç ağdaki standart SIP/UDP'ye çevirmeyi içerir.
    -   **Durum:** ⬜ Planlandı.