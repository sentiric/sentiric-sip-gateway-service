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
            handle_outbound_request(packet_str, sock, transactions, call_id).await;
        } else {
            handle_inbound_request(packet_str, remote_addr, sock, transactions, config, call_id, cseq_method).await;
        }
    } else {
        // =========================================================================
        //   DEĞİŞİKLİK BURADA: Loglama daha detaylı hale getirildi.
        // =========================================================================
        // Call-ID veya CSeq olmadan işlem yapamayız. Bu durum genellikle bir
        // ağ taraması (scan) veya bozuk bir pakettir. DEBUG seviyesinde loglayarak
        // hem sorunu tespit edebiliriz hem de production loglarını gereksiz yere doldurmayız.
        debug!(
            source = %remote_addr,
            packet_body = %packet_str,
            "Ayrıştırılamayan paket: Call-ID veya CSeq başlığı bulunamadı. Paket atlanıyor."
        );
        // =========================================================================
        //                               DEĞİŞİKLİK SONU
        // =========================================================================
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
    if packet_str.starts_with("INVITE") {
        info!("➡️ Gelen çağrı (INVITE) isteği alınıyor.");
    }

    if let Some(modified_packet) = processor::rewrite_inbound_request(packet_str, remote_addr, config) {
        if let (Some(via), Some(contact)) = (extract_header_value(packet_str, "Via"), extract_header_value(packet_str, "Contact")) {
            let mut guard = transactions.lock().await;
            guard.insert(
                (call_id, cseq_method),
                TransactionInfo {
                    original_client_addr: remote_addr,
                    original_via_header: via,
                    original_contact_header: contact,
                    created_at: Instant::now(),
                },
            );
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
    call_id: String,
) {
    if packet_str.starts_with("BYE") {
        info!("⬅️ Giden çağrı sonlandırma (BYE) isteği alınıyor.");
    }
    
    let guard = transactions.lock().await;
    if let Some(tx_info) = guard.get(&(call_id, "INVITE".to_string())) {
        let target_addr = tx_info.original_client_addr;
        
        let mut modified_packet = packet_str.to_string();
        if let Some(current_contact) = extract_header_value(&modified_packet, "Contact") {
             modified_packet = modified_packet.replacen(&current_contact, &tx_info.original_contact_header, 1);
        }
        
        debug!(%target_addr, "İstek telekom operatörüne yönlendiriliyor.");
        if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
            error!(error = %e, "Giden istek telekoma yönlendirilemedi.");
        }
    } else {
        warn!("Giden istekle eşleşen aktif INVITE işlemi bulunamadı.");
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
            info!("⬅️ Sinyal servisinden başarılı (200 OK) yanıtı alındı.");
        } else if let Some(code) = packet_str.split_whitespace().nth(1) {
             if code.starts_with('4') || code.starts_with('5') || code.starts_with('6') {
                 warn!(response_line = packet_str.lines().next().unwrap_or(""), "Sinyal servisinden hata yanıtı alındı.");
             }
        }
        
        let tx_key = (call_id, cseq_method.clone());
        let mut guard = transactions.lock().await;

        if let Some(tx_info) = guard.get(&tx_key) {
            let modified_packet = processor::rewrite_outbound_response(
                packet_str, 
                &tx_info.original_via_header, 
                config
            );

            if let Err(e) = sock.send_to(modified_packet.as_bytes(), tx_info.original_client_addr).await {
                error!(error = %e, "Yanıt istemciye yönlendirilemedi.");
            }

            if packet_str.contains(" 200 OK") && (cseq_method == "BYE" || cseq_method == "CANCEL") {
                info!("BYE/CANCEL işlemi tamamlandı, işlem kaydı siliniyor.");
                guard.remove(&tx_key);
            }
        } else {
            warn!("İşlem bulunamadı, yanıt yönlendirilemedi (muhtemelen eski bir BYE yanıtı).");
        }
    } else {
        warn!("Call-ID veya CSeq bulunamayan yanıt paketi geldi, atlanıyor.");
    }
}