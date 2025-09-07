// File: src/sip/processor.rs

use crate::config::AppConfig;
use crate::sip::message::SipMessage;
use crate::sip::transaction::TransactionInfo;
use std::net::SocketAddr;
use tracing::{instrument, warn};

/// Dış dünyadan (operatör) gelen bir isteği, iç ağdaki `signaling-service`'e
/// iletilecek temiz bir formata dönüştürür.
/// Bu fonksiyon, dış dünyanın karmaşık `Via` başlıklarını "yutar" ve yerine
/// iç ağda geçerli olan, sadece gateway'in bilgisini içeren TEK bir `Via` başlığı koyar.
#[instrument(name="rewrite_inbound", skip_all, fields(original_via_count = msg.via_headers.len()))]
pub fn rewrite_inbound_request(
    msg: &SipMessage,
    remote_addr: SocketAddr,
    config: &AppConfig,
) -> String {
    let mut new_lines = vec![msg.start_line.clone()];

    // Yeni ve tek Via başlığını oluştur.
    let branch = extract_branch_from_via(&msg.via_headers.first().cloned().unwrap_or_default()).unwrap_or_default();
    let new_via = format!(
        "Via: SIP/2.0/UDP {}:{};branch={};rport;received={}",
        config.public_ip,
        config.public_port,
        branch,
        remote_addr.ip()
    );
    new_lines.push(new_via);

    // DİKKAT: Orijinal Via başlıkları iç ağa GÖNDERİLMEZ. Topoloji gizlenir.
    
    // Diğer tüm başlıkları ve gövdeyi ekle.
    for (key, value) in &msg.headers {
        new_lines.push(format!("{}: {}", key, value));
    }
    
    new_lines.push(String::new()); // Başlık ve gövde arası boş satır
    new_lines.push(msg.body.clone());
    
    new_lines.join("\r\n") + "\r\n"
}

/// İç ağdaki `signaling-service`'ten gelen bir yanıtı, dış dünyadaki
/// orijinal istemciye (operatöre) gönderilecek doğru ve RFC uyumlu formata dönüştürür.
/// Bu fonksiyon, `signaling-service`'ten gelen yanıttaki basit, tek `Via` başlığını
/// atar ve yerine işlem başladığında kaydettiğimiz orijinal, çoklu `Via` listesini koyar.
#[instrument(name="rewrite_outbound", skip_all, fields(original_via_count = tx_info.original_via_headers.len()))]
pub fn rewrite_outbound_response(
    packet_str: &str,
    tx_info: &TransactionInfo,
    config: &AppConfig,
) -> String {
    let mut msg = match SipMessage::parse(packet_str) {
        Some(m) => m,
        None => return packet_str.to_string(),
    };

    // Gelen yanıttaki tüm Via'ları atıp, orijinal Via listesini koyuyoruz.
    msg.via_headers = tx_info.original_via_headers.clone();

    // Contact başlığını kendi public IP'mizle güncelliyoruz.
    if msg.headers.contains_key("Contact") {
        let new_contact = format!("<sip:gateway@{}:{}>", config.public_ip, config.public_port);
        msg.headers.insert("Contact".to_string(), new_contact);
    }

    // Server başlığını ekle/güncelle
    msg.headers.insert("Server".to_string(), format!("Sentiric Gateway v{}", config.service_version));
    
    // Mesajı yeniden birleştir
    let mut new_lines = vec![msg.start_line];
    new_lines.extend(msg.via_headers);
    for (key, value) in msg.headers {
        new_lines.push(format!("{}: {}", key, value));
    }
    new_lines.push(String::new());
    new_lines.push(msg.body);
    
    new_lines.join("\r\n") + "\r\n"
}

// --- Yardımcı Fonksiyonlar ---

pub fn extract_header_value(packet: &str, header_name: &str) -> Option<String> {
    packet
        .lines()
        .find(|line| line.trim().to_lowercase().starts_with(&format!("{}:", header_name.to_lowercase())))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_string())
}

pub fn extract_transaction_key(packet: &str) -> Option<(String, String)> {
    let call_id = extract_header_value(packet, "Call-ID")?;
    let cseq_line = extract_header_value(packet, "CSeq")?;
    let cseq_parts: Vec<&str> = cseq_line.split_whitespace().collect();
    if cseq_parts.len() == 2 {
        Some((call_id, cseq_parts[1].to_string()))
    } else {
        warn!(cseq_line = %cseq_line, "Geçersiz CSeq formatı");
        None
    }
}

fn extract_branch_from_via(via_header: &str) -> Option<String> {
    via_header.split(';').find(|part| part.trim().starts_with("branch="))
        .and_then(|part| part.split('=').nth(1))
        .map(|s| s.to_string())
}