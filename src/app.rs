// sentiric-sip-gateway-service/src/app.rs
use crate::config::AppConfig;
use crate::network;
use crate::sip;
use anyhow::{Context, Result};
use std::convert::Infallible;
use std::env;
use std::sync::Arc;
use tokio::select;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
// --- YENİ SATIRLAR ---
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
// --- BİTİŞ ---

pub struct App {
    config: Arc<AppConfig>,
}

// --- YENİ FONKSİYON ---
async fn health_check_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.uri().path() == "/healthz" {
        Ok(Response::new(Body::from(r#"{"status":"ok"}"#)))
    } else {
        let mut not_found = Response::default();
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
}
// --- BİTİŞ ---

impl App {
    pub async fn bootstrap() -> Result<Self> {
        // ... (Mevcut kod aynı kalıyor) ...
        dotenvy::dotenv().ok();
        let config = Arc::new(AppConfig::load_from_env().context("Konfigürasyon dosyası yüklenemedi")?);

        let rust_log_env = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,h2=warn,hyper=warn,tower=warn,rustls=warn".to_string());
        
        let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&rust_log_env))?;
        let subscriber = Registry::default().with(env_filter);
        
        if config.env == "development" {
            subscriber.with(fmt::layer().with_target(true).with_line_number(true)).init();
        } else {
            subscriber.with(fmt::layer().json().with_current_span(true).with_span_list(true)).init();
        }

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
        let cleanup_task = tokio::spawn(sip::transaction::cleanup_old_transactions(transactions.clone()));

        info!(listen_addr = %self.config.listen_addr, target_addr = %self.config.target_addr, "UDP dinleyici başlatılıyor...");
        let network_task = network::listen_and_process(self.config.clone(), transactions);

        // --- YENİ HTTP SUNUCUSU GÖREVİ ---
        let http_addr = format!("0.0.0.0:{}", self.config.http_port).parse()?;
        let http_server_task = tokio::spawn(async move {
            let make_svc = make_service_fn(|_conn| async {
                Ok::<_, Infallible>(service_fn(health_check_handler))
            });
            let server = Server::bind(&http_addr).serve(make_svc);
            info!(address = %http_addr, "HTTP sağlık kontrol sunucusu başlatıldı.");
            if let Err(e) = server.await {
                error!(error = %e, "HTTP sunucusu hatası.");
            }
        });
        // --- BİTİŞ ---

        select! {
            res = network_task => {
                if let Err(e) = res { error!(error = %e, "Kritik UDP ağ hatası."); }
            },
            res = http_server_task => {
                if let Err(e) = res { error!(error = %e, "Kritik HTTP sunucu hatası."); }
            },
            _ = signal::ctrl_c() => {
                warn!("Kapatma sinyali (Ctrl+C) alındı.");
            }
        }
        
        cleanup_task.abort();
        info!("✅ Servis başarıyla kapatıldı.");
        Ok(())
    }
}