// sentiric-sip-gateway-service/src/app.rs
use crate::config::AppConfig;
use crate::network;
use crate::sip;
use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use tokio::select;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

pub struct App {
    config: Arc<AppConfig>,
}

impl App {
    pub async fn bootstrap() -> Result<Self> {
        dotenvy::dotenv().ok();
        let config = Arc::new(AppConfig::load_from_env().context("Konfigürasyon dosyası yüklenemedi")?);

        // --- STANDARTLAŞTIRILMIŞ LOGLAMA KURULUMU ---
        let rust_log_env = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,h2=warn,hyper=warn,tower=warn,rustls=warn".to_string());
        
        let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&rust_log_env))?;
        let subscriber = Registry::default().with(env_filter);
        
        if config.env == "development" {
            subscriber.with(fmt::layer().with_target(true).with_line_number(true)).init();
        } else {
            subscriber.with(fmt::layer().json().with_current_span(true).with_span_list(true)).init();
        }
        // --- LOGLAMA KURULUMU SONU ---

        info!(
            service_name = "sentiric-sip-gateway-service",
            version = %config.service_version,
            commit = %config.git_commit,
            build_date = %config.build_date,
            profile = %config.env,
            "🚀 Servis başlatılıyor..."
        );
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        let transactions = sip::transaction::new_transaction_manager();
        
        // Periyodik olarak eski işlemleri temizleyen arka plan görevini başlat.
        let cleanup_task = tokio::spawn(sip::transaction::cleanup_old_transactions(transactions.clone()));

        info!(listen_addr = %self.config.listen_addr, target_addr = %self.config.target_addr, "UDP dinleyici başlatılıyor...");
        let network_task = network::listen_and_process(self.config.clone(), transactions);

        // Graceful shutdown mekanizması
        select! {
            res = network_task => {
                if let Err(e) = res {
                    error!(error = %e, "Kritik ağ hatası, servis durduruluyor.");
                }
            },
            _ = signal::ctrl_c() => {
                warn!("Kapatma sinyali (Ctrl+C) alındı. Servis gracefully kapatılıyor...");
            }
        }
        
        // Arka plan görevlerini iptal et.
        cleanup_task.abort();
        
        info!("✅ Servis başarıyla kapatıldı.");
        Ok(())
    }
}