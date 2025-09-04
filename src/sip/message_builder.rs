// File: src/sip/message_builder.rs
use crate::config::AppConfig;
use crate::sip::transaction::TransactionInfo;
use rand::Rng;

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
        if self.lines.is_empty() {
            return String::new();
        }
        let method = self.get_method();

        // 1. Route başlığını ekle (Request-URI'a dokunma!)
        self.add_route_header();

        // 2. Via başlığını sıfırdan oluştur.
        self.rewrite_via_header();

        // 3. Contact başlığını gateway'in public adresiyle değiştir.
        self.rewrite_contact_header();

        // 4. Max-Forwards'ı standart bir değere ayarla.
        self.set_header("Max-Forwards", "70");

        // 5. İçerik uzunluğunu garantile.
        self.ensure_content_length(&method);
        
        self.finalize()
    }

    fn get_method(&self) -> String {
        self.lines
            .first()
            .and_then(|line| line.split_whitespace().next())
            .map(String::from)
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn add_route_header(&mut self) {
        if let Some(record_route) = &self.invite_tx.record_route_header {
            let route_header = format!("Route: {}", record_route);
            self.lines.retain(|line| !line.to_lowercase().starts_with("route:"));
            self.lines.insert(1, route_header);
        }
    }
    
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
    
    fn set_header(&mut self, header_name: &str, value: &str) {
        let new_header = format!("{}: {}", header_name, value);
        self.replace_or_add_header(header_name, &new_header);
    }

    fn replace_or_add_header(&mut self, header_name: &str, new_header_line: &str) {
        if let Some(pos) = self.find_header_position(header_name) {
            self.lines[pos] = new_header_line.to_string();
        } else if let Some(pos) = self.find_header_position("cseq") {
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