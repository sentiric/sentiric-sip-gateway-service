// ========== FILE: sentiric-sip-gateway-service/src/main.rs ==========
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{error, info, instrument, warn};
use tracing_subscriber::EnvFilter;
use anyhow::{Context, Result}; // YENİ: Daha iyi hata yönetimi için

type Transactions = Arc<Mutex<HashMap<String, (SocketAddr, Instant)>>>;

// YENİ: Tüm konfigürasyonu tutacak yapı
#[derive(Debug)]
struct AppConfig {
    listen_addr: SocketAddr,
    target_addr: String,
    env: String,
}

// YENİ: Konfigürasyonu ortam değişkenlerinden yükleyen fonksiyon
fn load_config() -> Result<AppConfig> {
    // dotenvy kütüphanesi .env dosyasını bulamazsa hata vermez, bu yüzden güvenli.
    dotenvy::dotenv().ok();

    let env = env::var("ENV").unwrap_or_else(|_| "production".to_string());
    
    let listen_port = env::var("SIP_GATEWAY_SERVICE_PORT")
        .context("SIP_GATEWAY_SERVICE_PORT ortam değişkeni bulunamadı")?;
        
    let target_host = env::var("SIP_SIGNALING_SERVICE_HOST")
        .context("SIP_SIGNALING_SERVICE_HOST ortam değişkeni bulunamadı")?;
        
    let target_port = env::var("SIP_SIGNALING_SERVICE_PORT")
        .context("SIP_SIGNALING_SERVICE_PORT ortam değişkeni bulunamadı")?;

    let listen_addr_str = format!("0.0.0.0:{}", listen_port);
    let listen_addr = listen_addr_str.parse::<SocketAddr>()
        .with_context(|| format!("Geçersiz dinleme adresi: {}", listen_addr_str))?;
        
    let target_addr = format!("{}:{}", target_host, target_port);

    Ok(AppConfig {
        listen_addr,
        target_addr,
        env,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Önce konfigürasyonu yükle. Hata varsa program burada çöker.
    let config = load_config().expect("Konfigürasyon yüklenemedi");
    
    // 2. Sonra loglamayı kur.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(env_filter);

    if config.env == "development" {
        subscriber_builder.with_target(true).with_line_number(true).init();
    } else {
        subscriber_builder.json().with_current_span(true).with_span_list(true).init();
    }

    // 3. Servisi başlat.
    info!(config = ?config, "✅ SIP Gateway başlatılıyor...");

    let sock = Arc::new(UdpSocket::bind(config.listen_addr).await
        .with_context(|| format!("UDP porta bağlanılamadı: {}", config.listen_addr))?);
        
    let transactions: Transactions = Arc::new(Mutex::new(HashMap::new()));
    
    info!("Dinleme başladı.");

    let transactions_clone_for_cleanup = transactions.clone();
    tokio::spawn(cleanup_old_transactions(transactions_clone_for_cleanup));

    let mut buf = [0; 65535];
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, remote_addr)) => {
                 let packet_str = match std::str::from_utf8(&buf[..len]) {
                    Ok(s) => s,
                    Err(_) => {
                        warn!(source = %remote_addr, "UTF-8 olmayan bir paket alındı, atlanıyor.");
                        continue;
                    }
                };

                if packet_str.starts_with("SIP/2.0") {
                    handle_response_from_signaling(packet_str, &sock, &transactions).await;
                } else {
                    handle_request_from_client(packet_str, &sock, remote_addr, &config.target_addr, &transactions).await;
                }
            }
            Err(e) => {
                error!(error = %e, "UDP soketi okunurken hata oluştu.");
            }
        }
    }
}


#[instrument(skip_all, fields(source_addr = %remote_addr))]
async fn handle_request_from_client(
    packet_str: &str,
    sock: &UdpSocket,
    remote_addr: SocketAddr,
    target_addr: &str,
    transactions: &Transactions,
) {
    info!(packet_preview = %&packet_str[..packet_str.len().min(70)].replace("\r\n", " "), "➡️  İstemciden istek alındı.");
    if let Some(call_id) = extract_header_value(packet_str, "Call-ID") {
        let mut transactions_guard = transactions.lock().await;
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

#[instrument(skip_all, fields(call_id))]
async fn handle_response_from_signaling(packet_str: &str, sock: &UdpSocket, transactions: &Transactions) {
    if let Some(call_id) = extract_header_value(packet_str, "Call-ID") {
        tracing::Span::current().record("call_id", &call_id as &str);
        info!(packet_preview = %&packet_str[..packet_str.len().min(70)].replace("\r\n", " "), "⬅️  Sinyal servisinden yanıt alındı.");
        let transactions_guard = transactions.lock().await;
        if let Some((client_addr, _)) = transactions_guard.get(&call_id) {
            if let Err(e) = sock.send_to(packet_str.as_bytes(), client_addr).await {
                error!(error = %e, target_addr = %client_addr, "Yanıt istemciye yönlendirilemedi.");
            }
        } else {
            warn!("İşlem bulunamadı, yanıt yönlendirilemedi.");
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
            info!(cleaned_count = before_count - after_count, remaining_count = after_count, "Temizlik görevi: Eski işlemler temizlendi.");
        }
    }
}