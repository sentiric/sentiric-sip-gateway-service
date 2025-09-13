// File: src/sip/message_builder.rs

use crate::config::AppConfig;
use crate::sip::message::SipMessage;
use crate::sip::transaction::TransactionInfo;
use rand::Rng;
use tracing::instrument;

/// İç ağdan gelen bir isteği, dış dünyaya gönderilecek formata dönüştüren yapı.
pub struct OutboundRequestBuilder<'a> {
    msg: SipMessage,
    tx_info: &'a TransactionInfo,
    config: &'a AppConfig,
}

impl<'a> OutboundRequestBuilder<'a> {
    pub fn new(
        packet_str: &str,
        tx_info: &'a TransactionInfo,
        config: &'a AppConfig,
    ) -> Option<Self> {
        SipMessage::parse(packet_str).map(|msg| Self { msg, tx_info, config })
    }

    /// `BYE` veya `CANCEL` gibi diyalog içi bir isteği yeniden oluşturur.
    #[instrument(name="build_outbound_request", skip(self))]
    pub fn build(mut self) -> String {
        // 1. Route başlığını ekle (en kritik adım)
        // Saklanan Record-Route başlığını, Route başlığı olarak ekliyoruz.
        if let Some(record_route) = &self.tx_info.record_route_header {
            self.msg.headers.insert("Route".to_string(), record_route.clone());
        }

        // 2. Via başlığını yeniden yaz
        self.rewrite_via();

        // 3. Contact başlığını temizle (BYE/CANCEL'da olmamalı)
        self.msg.headers.remove("Contact");

        // 4. Max-Forwards'ı standart değere ayarla
        self.msg.headers.insert("Max-Forwards".to_string(), "70".to_string());

        // 5. User-Agent başlığını ekle/güncelle
        self.msg.headers.insert("User-Agent".to_string(), format!("Sentiric Gateway v{}", self.config.service_version));

        // 6. Content-Length'i sıfırla
        self.msg.headers.insert("Content-Length".to_string(), "0".to_string());

        // 7. Mesajı yeniden birleştir
        let mut new_lines = vec![self.msg.start_line];
        new_lines.extend(self.msg.via_headers); // Zaten tek bir tane olmalı
        for (key, value) in self.msg.headers {
            new_lines.push(format!("{}: {}", key, value));
        }
        new_lines.push(String::new()); // Boş satır

        new_lines.join("\r\n") + "\r\n"
    }

    fn rewrite_via(&mut self) {
        let branch: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let new_via = format!(
            "Via: SIP/2.0/UDP {}:{};branch=z9hG4bK.{}",
            self.config.public_ip, self.config.public_port, branch
        );
        // Var olan tek Via'yı bizimkiyle değiştiriyoruz.
        self.msg.via_headers = vec![new_via];
    }
}