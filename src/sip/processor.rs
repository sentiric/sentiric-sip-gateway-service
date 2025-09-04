// File: src/sip/processor.rs
use crate::config::AppConfig;
use crate::sip::message_builder::MessageBuilder; // YENİ: MessageBuilder'ı dahil et
use crate::sip::transaction::TransactionInfo;
use std::net::SocketAddr;
use tracing::warn;

pub fn rewrite_inbound_request(
    packet_str: &str,
    remote_addr: SocketAddr,
    config: &AppConfig,
) -> Option<String> {
    let original_via = extract_header_value(packet_str, "Via")?;
    
    let new_via = format!(
        "SIP/2.0/UDP {}:{};branch={};rport;received={}",
        config.public_ip,
        config.public_port,
        extract_branch_from_via(&original_via).unwrap_or_default(),
        remote_addr.ip()
    );

    Some(packet_str.replacen(&original_via, &new_via, 1))
}

// =========================================================================
//   SADELEŞTİRİLMİŞ FONKSİYON: Artık sadece MessageBuilder kullanıyor.
// =========================================================================
pub fn rewrite_outbound_request(
    packet_str: &str,
    invite_tx: &TransactionInfo,
    config: &AppConfig,
) -> String {
    MessageBuilder::new(packet_str, invite_tx, config)
        .build_outbound_request()
}
// =========================================================================
//                               DEĞİŞİKLİK SONU
// =========================================================================

pub fn rewrite_outbound_response(
    packet_str: &str,
    original_via: &str,
    config: &AppConfig
) -> String {
    let mut modified_packet = packet_str.to_string();

    if let Some(server_via) = extract_header_value(&modified_packet, "Via") {
        modified_packet = modified_packet.replacen(&server_via, original_via, 1);
    }
    
    if let Some(server_contact) = extract_header_value(&modified_packet, "Contact") {
         if let Some(user_part) = extract_user_from_uri(&server_contact) {
            let new_contact = format!("<sip:{}@{}:{}>", user_part, config.public_ip, config.public_port);
            modified_packet = modified_packet.replacen(&server_contact, &new_contact, 1);
        }
    }

    modified_packet
}

// --- Helper Fonksiyonlar (Değişiklik Yok) ---
pub fn extract_header_value(packet: &str, header_name: &str) -> Option<String> {
    let header_prefix_long = format!("{}:", header_name).to_lowercase();
    let header_prefix_short = format!("{}:", header_name.chars().next().unwrap().to_lowercase());
    packet
        .lines()
        .find(|line| {
            let trimmed_line = line.trim().to_lowercase();
            trimmed_line.starts_with(&header_prefix_long) ||
            (header_name != "CSeq" && trimmed_line.starts_with(&header_prefix_short))
        })
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

fn extract_user_from_uri(uri: &str) -> Option<String> {
    uri.split_once("sip:")
       .and_then(|(_, rest)| rest.split_once('@'))
       .map(|(user, _)| user.trim_start_matches('<').to_string())
}