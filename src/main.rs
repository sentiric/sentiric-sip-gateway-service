// File: src/main.rs (YENİ VE SADELEŞTİRİLMİŞ HALİ)
use std::process;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod config;
mod error;
mod network;
mod sip;

#[tokio::main]
async fn main() {
    let config = match config::AppConfig::load_from_env() {
        Ok(cfg) => Arc::new(cfg),
        Err(e) => {
            eprintln!("### BAŞLANGIÇ HATASI: Yapılandırma yüklenemedi: {}", e);
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
    
    info!(
        service_name = "sentiric-sip-gateway-service",
        version = %config.service_version,
        commit = %config.git_commit,
        build_date = %config.build_date,
        profile = %config.env,
        "🚀 Servis başlatılıyor..."
    );

    let transactions = sip::transaction::new_transaction_manager();
    
    // Temizlik görevini başlat
    tokio::spawn(sip::transaction::cleanup_old_transactions(transactions.clone()));

    info!(listen_addr = %config.listen_addr, target_addr = %config.target_addr, "UDP dinleyici başlatılıyor...");
    if let Err(e) = network::listen_and_process(config, transactions).await {
        error!(error = %e, "Kritik ağ hatası, servis durduruluyor.");
        process::exit(1);
    }
}