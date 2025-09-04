// File: src/sip/handler.rs
use crate::config::AppConfig;
use crate::sip::processor::{
    self, extract_header_value, extract_transaction_key,
};
use crate::sip::transaction::{TransactionInfo, Transactions};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, instrument, warn, Span};

#[instrument(level = "debug", skip_all, fields(source = %remote_addr, call_id, method))]
pub async fn handle_packet(
    packet_str: &str,
    remote_addr: SocketAddr,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    if packet_str.starts_with("SIP/2.0") {
        handle_response(packet_str, sock, transactions, config).await;
    } else {
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

    if let Some((call_id, cseq_method)) = extract_transaction_key(packet_str) {
        Span::current().record("call_id", &call_id as &str);
        
        let is_internal_request = match remote_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback(),
            IpAddr::V6(ipv6) => ipv6.is_loopback(),
        };

        if is_internal_request && remote_addr.port() != config.listen_addr.port() {
            handle_outbound_request(packet_str, sock, transactions, config, call_id, cseq_method).await;
        } else {
            handle_inbound_request(packet_str, remote_addr, sock, transactions, config, call_id, cseq_method).await;
        }
    } else {
        debug!(
            source = %remote_addr,
            packet_body = %packet_str,
            "Ayrıştırılamayan paket: Call-ID veya CSeq başlığı bulunamadı. Paket atlanıyor."
        );
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
            debug!("Yinelenen INVITE isteği alındı, atlanıyor.");
            return;
        }
        drop(guard);
        
        info!("➡️ Gelen çağrı (INVITE) isteği alınıyor.");
    }

    if let Some(modified_packet) = processor::rewrite_inbound_request(packet_str, remote_addr, config) {
        if cseq_method == "INVITE" {
            // Record-Route başlığını da alıyoruz.
            if let (Some(via), Some(contact)) = (extract_header_value(packet_str, "Via"), extract_header_value(packet_str, "Contact")) {
                let record_route = extract_header_value(packet_str, "Record-Route");
                
                let mut guard = transactions.lock().await;
                guard.insert(
                    (call_id, cseq_method),
                    TransactionInfo {
                        original_client_addr: remote_addr,
                        original_via_header: via,
                        original_contact_header: contact,
                        record_route_header: record_route, // Yeni alanı doldur
                        created_at: Instant::now(),
                    },
                );
            }
        }

        debug!("Paket modifiye edildi ve sinyal servisine yönlendiriliyor.");
        if let Err(e) = sock.send_to(modified_packet.as_bytes(), &config.target_addr).await {
            error!(error = %e, "Paket sinyal servisine yönlendirilemedi.");
        }
    } else {
        warn!("Gelen istek yeniden yazılamadı (başlıklar eksik olabilir).");
    }
}

async fn handle_outbound_request(
    packet_str: &str,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
    call_id: String,
    cseq_method: String,
) {
    if cseq_method == "BYE" {
        info!("⬅️ Giden çağrı sonlandırma (BYE) isteği alınıyor.");
    }
    
    let mut guard = transactions.lock().await;
    if let Some(invite_tx) = guard.get(&(call_id.clone(), "INVITE".to_string())).cloned() {
        
        let modified_packet = processor::rewrite_outbound_request(packet_str, &invite_tx, config);
        let target_addr = invite_tx.original_client_addr;
        
        guard.insert(
            (call_id, cseq_method),
            invite_tx,
        );
        
        drop(guard);

        debug!(%target_addr, "Modifiye edilmiş giden istek telekoma yönlendiriliyor.");
        if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
            error!(error = %e, "Giden istek telekoma yönlendirilemedi.");
        }
    } else {
        warn!("Giden istekle eşleşen aktif INVITE işlemi bulunamadı. İstek atlanıyor.");
    }
}

async fn handle_response(
    packet_str: &str,
    sock: &Arc<UdpSocket>,
    transactions: &Transactions,
    config: &Arc<AppConfig>,
) {
    if let Some((call_id, cseq_method)) = extract_transaction_key(packet_str) {
        Span::current().record("method", &cseq_method as &str);
        Span::current().record("call_id", &call_id as &str);
        
        if packet_str.contains(" 200 OK") {
            info!("⬅️ Sinyal servisinden veya telekomdan başarılı (200 OK) yanıtı alındı.");
        } else if let Some(code) = packet_str.split_whitespace().nth(1) {
             if code.starts_with('4') || code.starts_with('5') || code.starts_with('6') {
                 warn!(response_line = packet_str.lines().next().unwrap_or(""), "Hata yanıtı alındı.");
             }
        }
        
        let tx_key = (call_id, cseq_method.clone());
        let guard = transactions.lock().await;

        if let Some(tx_info) = guard.get(&tx_key) {
            let modified_packet = processor::rewrite_outbound_response(
                packet_str, 
                &tx_info.original_via_header, 
                config
            );
            let target_addr = tx_info.original_client_addr;

            drop(guard);

            if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
                error!(error = %e, "Yanıt istemciye yönlendirilemedi.");
            }
            
            if cseq_method == "BYE" || cseq_method == "CANCEL" {
                let mut guard = transactions.lock().await;
                info!("BYE/CANCEL işlemi tamamlandı, ilgili işlem kayıtları siliniyor.");
                guard.remove(&(tx_key.0.clone(), "INVITE".to_string()));
                guard.remove(&tx_key);
            }

        } else {
            warn!("İşlem bulunamadı, yanıt yönlendirilemedi (muhtemelen zaman aşımına uğramış bir işlem).");
        }
    } else {
        warn!("Call-ID veya CSeq bulunamayan yanıt paketi geldi, atlanıyor.");
    }
}