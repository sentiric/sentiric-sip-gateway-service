// DOSYA: sentiric-sip-gateway-service/src/main.rs (İSTEK/YANIT AYRIMLI NİHAİ VERSİYON)

use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{info, error, warn, instrument};
use tracing_subscriber::EnvFilter;

// Call-ID -> (istemci_adresi, oluşturulma_zamani)
type Transactions = Arc<Mutex<HashMap<String, (SocketAddr, Instant)>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().json().with_env_filter(env_filter).init();

    let listen_port = env::var("LISTEN_PORT").unwrap_or_else(|_| "5060".to_string());
    let target_host = env::var("TARGET_HOST").unwrap_or_else(|_| "sip-signaling".to_string());
    let target_port = env::var("TARGET_PORT").unwrap_or_else(|_| "5060".to_string());

    let listen_addr = format!("0.0.0.0:{}", listen_port);
    let target_addr = format!("{}:{}", target_host, target_port);

    let sock = Arc::new(UdpSocket::bind(&listen_addr).await?);
    let transactions: Transactions = Arc::new(Mutex::new(HashMap::new()));

    info!(listen_addr = %listen_addr, target_addr = %target_addr, "✅ SIP Gateway başlatıldı.");

    let transactions_clone_for_cleanup = transactions.clone();
    tokio::spawn(cleanup_old_transactions(transactions_clone_for_cleanup));

    let mut buf = [0; 65535];
    loop {
        let (len, remote_addr) = sock.recv_from(&mut buf).await?;
        let packet_str = match std::str::from_utf8(&buf[..len]) {
            Ok(s) => s,
            Err(_) => {
                warn!(source = %remote_addr, "UTF-8 olmayan bir paket alındı, atlanıyor.");
                continue;
            }
        };

        // --- KRİTİK DEĞİŞİKLİK: IP yerine paketin içeriğine göre karar veriyoruz ---
        if packet_str.starts_with("SIP/2.0") {
            // Bu bir yanıttır (örn: "SIP/2.0 200 OK")
            handle_response_from_signaling(packet_str, &sock, &transactions).await;
        } else {
            // Bu bir istektir (örn: "INVITE sip:...")
            handle_request_from_client(packet_str, &sock, remote_addr, &target_addr, &transactions).await;
        }
    }
}

#[instrument(skip_all)]
async fn handle_request_from_client(
    packet_str: &str,
    sock: &UdpSocket,
    remote_addr: SocketAddr,
    target_addr: &str,
    transactions: &Transactions,
) {
    info!(source = %remote_addr, "➡️  İstemciden istek alındı.");
    if let Some(call_id) = extract_header_value(packet_str, "Call-ID") {
        let mut transactions_guard = transactions.lock().await;
        // Sadece yeni INVITE'lar için transaction oluşturuyoruz.
        if packet_str.starts_with("INVITE") && !transactions_guard.contains_key(&call_id) {
            info!(%call_id, "Yeni bir çağrı için işlem kaydediliyor.");
            transactions_guard.insert(call_id.clone(), (remote_addr, Instant::now()));
        }

        if let Err(e) = sock.send_to(packet_str.as_bytes(), target_addr).await {
            error!(error = %e, "Paket sinyal servisine yönlendirilemedi.");
        }
    } else {
        warn!("Call-ID bulunamayan paket istemciden geldi, atlanıyor.");
    }
}

#[instrument(skip_all)]
async fn handle_response_from_signaling(packet_str: &str, sock: &UdpSocket, transactions: &Transactions) {
    info!("⬅️  Sinyal servisinden yanıt alındı.");
    if let Some(call_id) = extract_header_value(packet_str, "Call-ID") {
        let transactions_guard = transactions.lock().await;
        if let Some((client_addr, _)) = transactions_guard.get(&call_id) {
            if let Err(e) = sock.send_to(packet_str.as_bytes(), client_addr).await {
                error!(error = %e, "Yanıt istemciye yönlendirilemedi.");
            }
        } else {
            warn!(%call_id, "İşlem bulunamadı, yanıt yönlendirilemedi.");
        }
    } else {
        warn!("Call-ID bulunamayan paket sinyal servisinden geldi, atlanıyor.");
    }
}

fn extract_header_value(packet: &str, header_name: &str) -> Option<String> {
    let header_prefix_short = format!("{}:", header_name.chars().next().unwrap());
    let header_prefix_long = format!("{}:", header_name);

    packet.lines()
        .find(|line| {
            let lower_line = line.to_lowercase();
            lower_line.starts_with(&header_prefix_long.to_lowercase()) || lower_line.starts_with(&header_prefix_short.to_lowercase())
        })
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_string())
}

async fn cleanup_old_transactions(transactions: Transactions) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut guard = transactions.lock().await;
        let before_count = guard.len();
        guard.retain(|_call_id, (_addr, created_at)| created_at.elapsed() < Duration::from_secs(120));
        let after_count = guard.len();
        if before_count > after_count {
            info!("Temizlik görevi: {} eski işlem temizlendi. Kalan: {}", before_count - after_count, after_count);
        }
    }
}