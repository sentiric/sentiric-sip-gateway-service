// src/main.rs dosyasının TAM ve GÜNCELLENMİŞ HALİ
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{error, info, instrument, warn, Span};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct TransactionInfo {
    original_client_addr: SocketAddr,
    original_via_header: String,
    created_at: Instant,
}
type TransactionKey = (String, String);
type Transactions = Arc<Mutex<HashMap<TransactionKey, TransactionInfo>>>;

#[derive(Debug)]
struct AppConfig {
    pub listen_addr: SocketAddr,
    pub target_addr: String,
    pub env: String,
    // YENİ: Versiyon Bilgisi
    pub service_version: String,
    pub git_commit: String,
    pub build_date: String,
}

fn load_config() -> Result<AppConfig> {
    dotenvy::dotenv().ok();
    
    let env = env::var("ENV").unwrap_or_else(|_| "production".to_string());
    let listen_port = env::var("SIP_GATEWAY_LISTEN_PORT").unwrap_or_else(|_| "5060".to_string());
    let target_host = env::var("SIP_SIGNALING_SERVICE_HOST").context("ZORUNLU: SIP_SIGNALING_SERVICE_HOST ortam değişkeni eksik")?;
    let target_port = env::var("SIP_SIGNALING_SERVICE_PORT").context("ZORUNLU: SIP_SIGNALING_SERVICE_PORT ortam değişkeni eksik")?;
    let listen_addr_str = format!("0.0.0.0:{}", listen_port);
    let listen_addr = listen_addr_str.parse::<SocketAddr>().with_context(|| format!("Geçersiz dinleme adresi: {}", listen_addr_str))?;
    let target_addr = format!("{}:{}", target_host, target_port);

    // YENİ: Build-time environment değişkenlerini oku
    let service_version = env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let git_commit = env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build_date = env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string());


    Ok(AppConfig { 
        listen_addr, 
        target_addr, 
        env,
        service_version,
        git_commit,
        build_date,
    })
}

#[tokio::main]
async fn main() {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("### KONFİGÜRASYON HATASI: {:?}", e);
            process::exit(1);
        }
    };
    
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(env_filter);

    if config.env == "development" {
        subscriber_builder.with_target(true).with_line_number(true).init();
    } else {
        subscriber_builder.json().with_current_span(true).with_span_list(true).init();
    }
    
    // YENİ: Başlangıçta versiyon bilgisini logla
    info!(
        service_name = "sentiric-sip-gateway-service",
        version = %config.service_version,
        commit = %config.git_commit,
        build_date = %config.build_date,
        profile = %config.env,
        "🚀 Servis başlatılıyor..."
    );

    let sock = Arc::new(UdpSocket::bind(config.listen_addr).await.unwrap_or_else(|e| {
        error!(address = %config.listen_addr, error = %e, "UDP porta bağlanılamadı.");
        process::exit(1);
    }));
        
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
                    Err(_) => { warn!(source = %remote_addr, "UTF-8 olmayan bir paket alındı, atlanıyor."); continue; }
                };
                if packet_str.starts_with("SIP/2.0") {
                    handle_response_from_signaling(packet_str, &sock, &transactions).await;
                } else {
                    handle_request_from_client(packet_str, &sock, remote_addr, &config.target_addr, &transactions, &config).await;
                }
            }
            Err(e) => { error!(error = %e, "UDP soketi okunurken hata oluştu."); }
        }
    }
}

#[instrument(skip_all, fields(source_addr = %remote_addr, call_id, method))]
async fn handle_request_from_client(
    packet_str: &str,
    sock: &UdpSocket,
    remote_addr: SocketAddr,
    target_addr: &str,
    transactions: &Transactions,
    config: &AppConfig,
) {
    info!(packet_preview = %&packet_str[..packet_str.len().min(70)].replace("\r\n", " "), "➡️  İstemciden istek alındı.");
    
    if let Some((call_id, cseq_method)) = extract_transaction_key(packet_str) {
        Span::current().record("call_id", &call_id as &str);
        Span::current().record("method", &cseq_method as &str);
        
        if let Some(original_via) = extract_header_value(packet_str, "Via") {
            let mut transactions_guard = transactions.lock().await;
            let tx_key = (call_id, cseq_method);
    
            if !transactions_guard.contains_key(&tx_key) {
                info!("Yeni bir SIP işlemi için kayıt oluşturuluyor.");
                transactions_guard.insert(tx_key, TransactionInfo {
                    original_client_addr: remote_addr,
                    original_via_header: original_via.clone(),
                    created_at: Instant::now(),
                });
            }

            // --- DEĞİŞİKLİK BURADA: `sip_listen_addr` -> `listen_addr` ve `to_string()` eklendi ---
            let new_via = format!(
                "SIP/2.0/UDP {}:{};branch={};rport;received={}",
                config.listen_addr.ip(),
                config.listen_addr.port(),
                extract_branch_from_via(&original_via).unwrap_or_else(|| "sentiric-gateway".to_string()),
                remote_addr.ip()
            );
            // --- DEĞİŞİKLİK SONU ---
            
            let modified_packet = packet_str.replacen(&original_via, &new_via, 1);
    
            if let Err(e) = sock.send_to(modified_packet.as_bytes(), target_addr).await {
                error!(error = %e, "Modifiye edilmiş paket sinyal servisine yönlendirilemedi.");
            }
        } else {
            warn!("Via başlığı olmayan paket geldi, atlanıyor.");
        }
    } else {
        warn!("Call-ID veya CSeq bulunamayan paket istemciden geldi, atlanıyor.");
    }
}

#[instrument(skip_all, fields(call_id, method))]
async fn handle_response_from_signaling(packet_str: &str, sock: &UdpSocket, transactions: &Transactions) {
    if let Some((call_id, cseq_method)) = extract_transaction_key(packet_str) {
        Span::current().record("call_id", &call_id as &str);
        Span::current().record("method", &cseq_method as &str);

        info!(packet_preview = %&packet_str[..packet_str.len().min(70)].replace("\r\n", " "), "⬅️  Sinyal servisinden yanıt alındı.");
        let transactions_guard = transactions.lock().await;
        
        let tx_key = (call_id, cseq_method);
        if let Some(tx_info) = transactions_guard.get(&tx_key) {
            if let Some(server_via) = extract_header_value(packet_str, "Via") {
                let modified_packet = packet_str.replacen(&server_via, &tx_info.original_via_header, 1);
                if let Err(e) = sock.send_to(modified_packet.as_bytes(), tx_info.original_client_addr).await {
                    error!(error = %e, target_addr = %tx_info.original_client_addr, "Yanıt istemciye yönlendirilemedi.");
                }
            } else {
                warn!("Yanıtta Via başlığı yok, paket değiştirilemedi.");
            }
        } else {
            warn!("İşlem bulunamadı, yanıt yönlendirilemedi.");
        }
    } else {
        warn!("Call-ID veya CSeq bulunamayan paket sinyal servisinden geldi, atlanıyor.");
    }
}

fn extract_branch_from_via(via_header: &str) -> Option<String> {
    via_header.split(';').find(|part| part.trim().starts_with("branch="))
        .and_then(|part| part.split('=').nth(1))
        .map(|s| s.to_string())
}

fn extract_header_value(packet: &str, header_name: &str) -> Option<String> {
    let header_prefix_long = format!("{}:", header_name);
    packet.lines()
        .find(|line| line.to_lowercase().starts_with(&header_prefix_long.to_lowercase()))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_string())
}

fn extract_transaction_key(packet: &str) -> Option<(String, String)> {
    let call_id = extract_header_value(packet, "Call-ID")?;
    let cseq_line = extract_header_value(packet, "CSeq")?;
    let cseq_parts: Vec<&str> = cseq_line.split_whitespace().collect();
    if cseq_parts.len() == 2 {
        Some((call_id, cseq_parts[1].to_string()))
    } else {
        None
    }
}

async fn cleanup_old_transactions(transactions: Transactions) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut guard = transactions.lock().await;
        let before_count = guard.len();
        guard.retain(|_key, tx_info| tx_info.created_at.elapsed() < Duration::from_secs(120));
        let after_count = guard.len();
        if before_count > after_count {
            info!(cleaned_count = before_count - after_count, remaining_count = after_count, "Temizlik görevi: Eski işlemler temizlendi.");
        }
    }
}