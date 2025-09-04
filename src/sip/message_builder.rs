// File: src/sip/message_builder.rs (YENİ DOSYA)
use crate::config::AppConfig;
use crate::sip::transaction::TransactionInfo;

/// SIP mesajlarını programatik olarak oluşturmak ve değiştirmek için bir yardımcı yapı.
pub struct MessageBuilder<'a> {
    lines: Vec<String>,
    invite_tx: &'a TransactionInfo,
    config: &'a AppConfig,
}

impl<'a> MessageBuilder<'a> {
    /// Mevcut bir SIP paketinden yeni bir MessageBuilder oluşturur.
    pub fn new(packet_str: &str, invite_tx: &'a TransactionInfo, config: &'a AppConfig) -> Self {
        Self {
            lines: packet_str.lines().map(String::from).collect(),
            invite_tx,
            config,
        }
    }

    /// Giden bir isteği (örn: BYE) yeniden oluşturur.
    pub fn build_outbound_request(mut self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }

        // Metodu, 'lines' a olan bağımlılıktan kurtarıyoruz.
        let method = self
            .lines
            .first()
            .and_then(|line| line.split_whitespace().next())
            .map(String::from)
            .unwrap_or_else(|| "UNKNOWN".to_string());

        self.rewrite_request_uri(&method);
        self.rewrite_contact_header();
        self.ensure_content_length(&method);
        
        self.finalize()
    }

    /// Request-URI'ı (ilk satır) orijinal INVITE'taki Contact ile değiştirir.
    fn rewrite_request_uri(&mut self, method: &str) {
        self.lines[0] = format!("{} {} SIP/2.0", method, self.invite_tx.original_contact_header);
    }

    /// Contact başlığını gateway'in public IP adresiyle yeniden yazar veya ekler.
    fn rewrite_contact_header(&mut self) {
        let new_contact = format!("Contact: <sip:sentiric@{}:{}>", self.config.public_ip, self.config.public_port);
        if let Some(pos) = self.find_header_position("contact") {
            self.lines[pos] = new_contact;
        } else if let Some(pos) = self.find_header_position("cseq") {
            self.lines.insert(pos + 1, new_contact);
        }
    }

    /// BYE gibi gövdesiz mesajlar için Content-Length: 0 başlığını garantiler.
    fn ensure_content_length(&mut self, method: &str) {
        if method == "BYE" || method == "CANCEL" {
            let content_length = "Content-Length: 0".to_string();
            if let Some(pos) = self.find_header_position("content-length") {
                self.lines[pos] = content_length;
            } else if let Some(pos) = self.find_header_position("cseq") {
                self.lines.insert(pos + 1, content_length);
            }
        }
    }

    /// Belirtilen başlığın satır indeksini bulur.
    fn find_header_position(&self, header_name: &str) -> Option<usize> {
        let prefix = format!("{}:", header_name);
        self.lines.iter().position(|l| l.to_lowercase().starts_with(&prefix))
    }

    /// Mesaj satırlarını standart SIP formatında birleştirir.
    fn finalize(self) -> String {
        self.lines.join("\r\n") + "\r\n\r\n"
    }
}