// File: sentiric-sip-gateway-service/src/sip/handler.rs
use crate::config::AppConfig;
use crate::sip::processor::{self, extract_transaction_key};
use crate::sip::transaction::{TransactionInfo, Transactions};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, instrument, warn, Span}; // instrument'ı import et

// DÜZELTME: Artık #[instrument] kullanıyoruz.
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
    if let Some((call_id, cseq_line)) = processor::extract_full_transaction_key(packet_str) {
        Span::current().record("call_id", &call_id);
        Span::current().record("cseq", &cseq_line);
    }

    if packet_str.starts_with("SIP/2.0") {
        Span::current().record("direction", "response");
        handle_response(packet_str, remote_addr, sock, transactions, config).await;
    } else {
        Span::current().record("direction", "request");
        handle_request(packet_str, remote_addr, sock, transactions, config).await;
    }
}

async fn handle_request(
    packet_str: &str,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    let method = packet_str.split_whitespace().next().unwrap_or("UNKNOWN");
    Span::current().record("method", &method);

    let is_internal_request = match remote_addr.ip() {
        IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    };
    
    // Gelen isteğin anahtarlarını tekrar çıkarmaya gerek yok, span'e zaten eklendi.
    let (call_id, cseq_method) = extract_transaction_key(packet_str).unwrap_or_default();

    if is_internal_request && remote_addr.port() != config.listen_addr.port() {
        info!("⬅️ Giden istek alındı (internal -> external)");
        handle_outbound_request(packet_str, sock, transactions, config, call_id, cseq_method).await;
    } else {
        info!("➡️ Gelen istek alındı (external -> internal)");
        handle_inbound_request(packet_str, remote_addr, sock, transactions, config, call_id, cseq_method).await;
    }
}


async fn handle_inbound_request(
    packet_str: &str,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
    call_id: String,
    cseq_method: String,
) {
    if cseq_method == "INVITE" {
        let guard = transactions.lock().await;
        if guard.contains_key(&(call_id.clone(), cseq_method.clone())) {
            debug!("Yinelenen INVITE isteği, atlanıyor.");
            return;
        }
        drop(guard);
    }

    if let Some(modified_packet) = processor::rewrite_inbound_request(packet_str, remote_addr, config) {
        if cseq_method == "INVITE" {
            if let (Some(via), Some(contact)) = (processor::extract_header_value(packet_str, "Via"), processor::extract_header_value(packet_str, "Contact")) {
                let record_route = processor::extract_header_value(packet_str, "Record-Route");
                let mut guard = transactions.lock().await;
                guard.insert(
                    (call_id, cseq_method),
                    TransactionInfo {
                        original_client_addr: remote_addr,
                        original_via_header: via,
                        original_contact_header: contact,
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
    } else {
        warn!("Gelen istek yeniden yazılamadı (başlıklar eksik olabilir).");
    }
}


async fn handle_outbound_request(
    packet_str: &str,
    sock: &Arc<UpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
    call_id: String,
    cseq_method: String,
) {
    let mut guard = transactions.lock().await;
    if let Some(invite_tx) = guard.get(&(call_id.clone(), "INVITE".to_string())).cloned() {
        let modified_packet = processor::rewrite_outbound_request(packet_str, &invite_tx, config);
        let target_addr = invite_tx.original_client_addr;
        guard.insert((call_id, cseq_method), invite_tx);
        drop(guard);
        debug!(to = %target_addr, "Modifiye edilmiş giden istek telekoma yönlendiriliyor.");
        if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
            error!(error = %e, target = %target_addr, "Giden istek telekoma yönlendirilemedi.");
        }
    } else {
        warn!(call_id = %call_id, "Giden istekle eşleşen aktif INVITE işlemi bulunamadı. İstek atlanıyor.");
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
        
        if response_line.contains(" 200 OK") {
            info!("⬅️ Başarılı yanıt (200 OK) alındı.");
        } else if let Some(code) = response_line.split_whitespace().nth(1) {
             if code.starts_with('4') || code.starts_with('5') || code.starts_with('6') {
                 warn!(response_line = %response_line, "Hata yanıtı alındı.");
             }
        }
        
        let tx_key = (call_id, cseq_method.clone());
        let guard = transactions.lock().await;
        if let Some(tx_info) = guard.get(&tx_key) {
            let modified_packet = processor::rewrite_outbound_response(packet_str, &tx_info.original_via_header, config);
            let target_addr = tx_info.original_client_addr;
            drop(guard);
            if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
                error!(error = %e, "Yanıt istemciye yönlendirilemedi.");
            }
            if cseq_method == "BYE" || cseq_method == "CANCEL" || response_line.contains(" 4") {
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