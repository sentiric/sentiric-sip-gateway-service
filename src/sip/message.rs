// File: src/sip/message.rs

use std::collections::HashMap;

/// SIP mesajının ayrıştırılmış halini temsil eden yapı.
/// Bu yapı, SIP mesajlarını daha güvenli ve kolay bir şekilde işlememizi sağlar.
#[derive(Debug, Clone)]
pub struct SipMessage {
    pub start_line: String,
    pub headers: HashMap<String, String>,
    pub via_headers: Vec<String>, // Birden fazla Via başlığını sırasıyla saklamak için
    pub body: String,
}

impl SipMessage {
    /// Ham metin bir paketten yeni bir SipMessage nesnesi oluşturur.
    /// Operatörlerden gelen çoklu 'Via' başlıklarını doğru bir şekilde ayrıştırır.
    pub fn parse(packet_str: &str) -> Option<Self> {
        let mut lines = packet_str.lines();
        let start_line = lines.next()?.to_string();

        let mut headers = HashMap::new();
        let mut via_headers = Vec::new();
        let mut body_lines = Vec::new();
        let mut in_body = false;

        for line in lines {
            if in_body {
                body_lines.push(line);
                continue;
            }
            if line.is_empty() {
                in_body = true;
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim().to_string();
                let key_lower = key_trimmed.to_lowercase();

                // 'Via' başlıklarını (ve kısa formu 'v') özel olarak ele alıp vektöre ekliyoruz.
                // Başlığın tamamını ("Via: ...") koruyoruz ki yanıtta aynen geri gönderebilelim.
                if key_lower == "via" || key_lower == "v" {
                    via_headers.push(line.to_string());
                } else {
                    headers.insert(key_trimmed.to_string(), value_trimmed);
                }
            }
        }

        Some(SipMessage {
            start_line,
            headers,
            via_headers,
            body: body_lines.join("\r\n"),
        })
    }
}