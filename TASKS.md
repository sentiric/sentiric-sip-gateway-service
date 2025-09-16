# 🛡️ Sentiric SIP Gateway Service - Görev Listesi (v3.1 - Stabilite)

Bu belge, `sip-gateway-service`'in geliştirme yol haritasını, tamamlanan önemli kilometre taşlarını ve gelecekteki hedeflerini tanımlar.

---

### **FAZ 1: Kritik Mantık Hatasını Giderme (Mevcut Odak)**

-   **Görev ID: GW-BUG-01 - Giden `BYE` İsteği Oluşturma Mantığını Düzelt**
    -   **Durum:** x **Yapılacak (Öncelik 1 - KRİTİK)**
    -   **Problem:** `agent-service` tarafından tetiklenen bir çağrı sonlandırma senaryosunda, bu servis iç ağdan gelen `BYE` isteğini telekom operatörüne göndermek için yeniden oluştururken, `SipMessage::parse` fonksiyonuna sadece isteğin başlangıç satırını ("BYE sip:...") vermektedir. Bu, `Call-ID`, `CSeq` gibi kritik başlıkların kaybolmasına ve geçersiz bir SIP paketinin üretilmesine neden olur. Bu hata, platformun bir çağrıyı proaktif olarak sonlandırmasını engeller.
    -   **Çözüm:**
        -   [x] `src/sip/handler.rs` içindeki `handle_request` fonksiyonu, aldığı orijinal `packet_str`'ı `handle_outbound_request` fonksiyonuna parametre olarak geçirmelidir.
        -   [x] `handle_outbound_request` fonksiyonu, `OutboundRequestBuilder::new` metodunu çağırırken artık `&msg.start_line` yerine bu tam `packet_str`'ı kullanmalıdır. Bu, `SipMessage`'in doğru şekilde ayrıştırılmasını ve tüm gerekli başlıkların korunmasını sağlayacaktır.

---
### **Gelecek Fazlar**

-   [ ] **Görev ID: GW-PROTO-001 - WebRTC Entegrasyonu (SIP over WebSocket)**       
-   [ ] **Görev ID: GW-SEC-001 - Hız Sınırlama (Rate Limiting)**
-   [ ] **Görev ID: GW-SEC-002 - IP Beyaz/Kara Liste**
-   [ ] **Görev ID: GW-OBSERV-001 - Prometheus Metrikleri**
 