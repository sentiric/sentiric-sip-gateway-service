// File: src/sip/message_builder.rs (TAM KOD)
use crate::config::AppConfig;
use crate::sip::transaction::TransactionInfo;
use rand::Rng; // YENİ: Rastgele branch değeri için

/// SIP mesajlarını programatik olarak oluşturmak ve değiştirmek için bir yardımcı yapı.
pub struct MessageBuilder<'a> {
    lines: Vec<String>,
    invite_tx: &'a TransactionInfo,
    config: &'a AppConfig,
}

impl<'a> MessageBuilder<'a> {
    pub fn new(packet_str: &str, invite_tx: &'a TransactionInfo, config: &'a AppConfig) -> Self {
        Self {
            lines: packet_str.lines().map(String::from).collect(),
            invite_tx,
            config,
        }
    }

    pub fn build_outbound_request(mut self) -> String {
        if self.lines.is_empty() { return String::new(); }

        let method = self.get_method();
        
        // =========================================================================
        //   DEĞİŞİKLİK BURADA: Mantığı tamamen yeniliyoruz.
        // =========================================================================
        // sip-signaling'den gelen paketi temel alıp sadece gerekli başlıkları değiştiriyoruz.
        // Bu, To, From, Call-ID gibi diyaloğa özgü bilgilerin korunmasını sağlar.
        
        // 1. Request-URI'ı orijinal Contact ile değiştir.
        self.rewrite_request_uri(&method);

        // 2. Via başlığını sıfırdan, sadece kendi bilgimizle oluştur.
        //    Bu, proxy zinciri sorununu çözer.
        self.rewrite_via_header();
        
        // 3. Contact başlığını gateway'in public adresiyle değiştir.
        self.rewrite_contact_header();

        // 4. Max-Forwards'ı standart bir değere ayarla.
        self.set_header("Max-Forwards", "70");
        
        self.ensure_content_length(&method);
        // =========================================================================
        //                               DEĞİŞİKLİK SONU
        // =========================================================================
        
        self.finalize()
    }

    fn get_method(&self) -> String {
        self.lines
            .first()
            .and_then(|line| line.split_whitespace().next())
            .map(String::from)
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }
    
    fn rewrite_request_uri(&mut self, method: &str) {
        self.lines[0] = format!("{} {} SIP/2.0", method, self.invite_tx.original_contact_header);
    }
    
    // YENİ FONKSİYON
    fn rewrite_via_header(&mut self) {
        let branch: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let new_via = format!(
            "Via: SIP/2.0/UDP {}:{};branch=z9hG4bK.{}", 
            self.config.public_ip, 
            self.config.public_port, 
            branch
        );
        self.replace_or_add_header("via", &new_via);
    }

    fn rewrite_contact_header(&mut self) {
        let new_contact = format!("Contact: <sip:sentiric@{}:{}>", self.config.public_ip, self.config.public_port);
        self.replace_or_add_header("contact", &new_contact);
    }

    fn ensure_content_length(&mut self, method: &str) {
        if method == "BYE" || method == "CANCEL" {
            self.set_header("Content-Length", "0");
        }
    }
    
    // YENİ HELPER FONKSİYON
    fn set_header(&mut self, header_name: &str, value: &str) {
        let new_header = format!("{}: {}", header_name, value);
        self.replace_or_add_header(header_name, &new_header);
    }

    // YENİ HELPER FONKSİYON
    fn replace_or_add_header(&mut self, header_name: &str, new_header_line: &str) {
        if let Some(pos) = self.find_header_position(header_name) {
            self.lines[pos] = new_header_line.to_string();
        } else if let Some(pos) = self.find_header_position("cseq") {
            // CSeq'ten sonra eklemek mantıklı bir varsayım.
            self.lines.insert(pos + 1, new_header_line.to_string());
        }
    }

    fn find_header_position(&self, header_name: &str) -> Option<usize> {
        let prefix = format!("{}:", header_name).to_lowercase();
        self.lines.iter().position(|l| l.to_lowercase().starts_with(&prefix))
    }

    fn finalize(self) -> String {
        self.lines.join("\r\n") + "\r\n\r\n"
    }
}