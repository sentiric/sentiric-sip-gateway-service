// File: src/sip/handler.rs

use crate::config::AppConfig;
use crate::sip::message::SipMessage;
use crate::sip::message_builder::OutboundRequestBuilder; // YENİ
use crate::sip::processor::{self, extract_transaction_key};
use crate::sip::transaction::{TransactionInfo, Transactions};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, instrument, warn, Span};

#[instrument(
    name = "sip_packet",
    level = "info",
    skip_all,
    fields(
        source = %remote_addr,
        call_id = tracing::field::Empty,
        cseq = tracing::field::Empty,
        method = tracing::field::Empty,
        direction = tracing::field::Empty
    )
)]
pub async fn handle_packet(
    packet_str: &str,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let msg = match SipMessage::parse(packet_str) {
        Some(m) => m,
        None => {
            warn!("Gelen SIP paketi ayrıştırılamadı, atlanıyor.");
            return;
        }
    };
    
    if let Some(call_id) = msg.headers.get("Call-ID") {
        Span::current().record("call_id", call_id);
    }
    if let Some(cseq) = msg.headers.get("CSeq") {
        Span::current().record("cseq", cseq);
    }

    if msg.start_line.starts_with("SIP/2.0") {
        Span::current().record("direction", "response");
        handle_response(packet_str, remote_addr, sock, transactions, config).await;
    } else {
        let method = msg.start_line.split_whitespace().next().unwrap_or("UNKNOWN");
        Span::current().record("method", method);
        Span::current().record("direction", "request");
        handle_request(&msg, remote_addr, sock, transactions, config).await;
    }
}

async fn handle_request(
    msg: &SipMessage,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let is_internal_request = match SocketAddr::from_str(&config.target_addr) {
        Ok(target_socket_addr) => remote_addr == target_socket_addr,
        Err(_) => false,
    };

    if is_internal_request {
        info!("⬅️ Giden istek alındı (internal -> external)");
        // --- GÜNCELLEME BAŞLANGICI: Artık bu fonksiyonu çağırıyoruz. ---
        handle_outbound_request(msg, sock, transactions, config).await;
        // --- GÜNCELLEME SONU ---
    } else {
        info!("➡️ Gelen istek alındı (external -> internal)");
        handle_inbound_request(msg, remote_addr, sock, transactions, config).await;
    }
}

// --- YENİ EKLENEN/TAMAMLANAN FONKSİYON ---
async fn handle_outbound_request(
    msg: &SipMessage,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let call_id = msg.headers.get("Call-ID");
    let cseq_method = msg.start_line.split_whitespace().nth(0).unwrap_or_default();

    if call_id.is_none() {
        warn!("İç servisten Call-ID'siz giden istek geldi, atlanıyor.");
        return;
    }
    let call_id_str = call_id.unwrap();

    // Bu giden isteğin ait olduğu orijinal INVITE işlemini bulmalıyız.
    let guard = transactions.lock().await;
    if let Some(invite_tx) = guard.get(&(call_id_str.clone(), "INVITE".to_string())).cloned() {
        drop(guard); // Kilidi erken bırak

        let packet_str = &msg.start_line; // Sadece bir örnek, builder tüm mesajı almalı
        
        // OutboundRequestBuilder'ı kullanarak isteği yeniden oluştur
        if let Some(builder) = OutboundRequestBuilder::new(packet_str, &invite_tx, config) {
            let modified_packet = builder.build();
            let target_addr = invite_tx.original_client_addr;

            debug!(to = %target_addr, "Modifiye edilmiş giden istek operatöre yönlendiriliyor.");
            if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
                error!(error = %e, target = %target_addr, "Giden istek operatöre yönlendirilemedi.");
            }
        } else {
            error!("Giden istek için SipMessage parse edilemedi.");
        }
    } else {
        warn!(call_id = %call_id_str, method = %cseq_method, "Giden istekle eşleşen aktif INVITE işlemi bulunamadı. İstek atlanıyor.");
    }
}
// --- YENİ EKLENEN/TAMAMLANAN FONKSİYON SONU ---

async fn handle_inbound_request(
    msg: &SipMessage,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let method = msg.start_line.split_whitespace().next().unwrap_or_default();
    
    if method == "INVITE" {
        if let Some(call_id) = msg.headers.get("Call-ID") {
            let guard = transactions.lock().await;
            if guard.contains_key(&(call_id.clone(), "INVITE".to_string())) {
                debug!("Yinelenen INVITE isteği, atlanıyor.");
                return;
            }
        }
    }

    let modified_packet = processor::rewrite_inbound_request(msg, remote_addr, config);
    
    if method == "INVITE" {
        if let (Some(contact), Some(call_id), Some(_cseq)) = (msg.headers.get("Contact"), msg.headers.get("Call-ID"), msg.headers.get("CSeq")) {
            let record_route = msg.headers.get("Record-Route").cloned();
            let mut guard = transactions.lock().await;
            guard.insert(
                (call_id.clone(), "INVITE".to_string()),
                TransactionInfo {
                    original_client_addr: remote_addr,
                    original_via_headers: msg.via_headers.clone(),
                    original_contact_header: contact.clone(),
                    record_route_header: record_route,
                    created_at: Instant::now(),
                },
            );
        }
    }
    
    debug!(to = %config.target_addr, "Paket sinyal servisine yönlendiriliyor.");
    if let Err(e) = sock.send_to(modified_packet.as_bytes(), &config.target_addr).await {
        error!(error = %e, target = %config.target_addr, "Paket sinyal servisine yönlendirilemedi.");
    }
}



async fn handle_response(
    packet_str: &str,
    _remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let response_line = packet_str.lines().next().unwrap_or("");
    if let Some((call_id, cseq_method)) = extract_transaction_key(packet_str) {
        Span::current().record("method", &cseq_method as &str);
        
        let tx_key = (call_id, cseq_method.clone());
        let guard = transactions.lock().await;
        if let Some(tx_info) = guard.get(&tx_key) {
            let modified_packet = processor::rewrite_outbound_response(packet_str, tx_info, config);
            let target_addr = tx_info.original_client_addr;
            drop(guard);
            if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
                error!(error = %e, "Yanıt istemciye yönlendirilemedi.");
            }
            if cseq_method == "BYE" || cseq_method == "CANCEL" || response_line.contains(" 4") || response_line.contains(" 5") || response_line.contains(" 6") {
                let mut guard = transactions.lock().await;
                debug!("İşlem tamamlandı, ilgili kayıtlar siliniyor.");
                guard.remove(&(tx_key.0.clone(), "INVITE".to_string()));
                guard.remove(&tx_key);
            }
        } else {
            debug!("İşlem bulunamadı, yanıt yönlendirilemedi (muhtemelen zaman aşımına uğramış bir işlem).");
        }
    } else {
        warn!("Call-ID veya CSeq bulunamayan yanıt paketi geldi, atlanıyor.");
    }
}