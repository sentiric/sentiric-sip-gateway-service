// sentiric-sip-gateway-service/src/app.rs
use crate::config::AppConfig;
use crate::network;
use crate::sip;
use anyhow::{Context, Result};
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::select;
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

pub struct App {
    config: Arc<AppConfig>,
}

async fn health_check_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(r#"{"status":"ok"}"#)))
}

fn spawn_http_server(config: Arc<AppConfig>) -> (JoinHandle<()>, tokio::sync::oneshot::Sender<()>) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], config.http_port));
        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(health_check_handler))
        });

        let server = Server::bind(&addr)
            .serve(make_svc)
            .with_graceful_shutdown(async {
                rx.await.ok();
            });

        info!(address = %addr, "HTTP sağlık kontrol sunucusu başlatıldı.");
        if let Err(e) = server.await {
            error!(error = %e, "HTTP sunucusu hatası.");
        }
    });
    (handle, tx)
}

impl App {
    pub async fn bootstrap() -> Result<Self> {
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

        let (http_server_handle, http_shutdown_tx) = spawn_http_server(self.config.clone());
        let network_task = network::listen_and_process(self.config.clone(), transactions);

        select! {
            res = network_task => {
                if let Err(e) = res { error!(error = %e, "Kritik UDP ağ hatası."); }
            },
            res = http_server_handle => {
                if let Err(e) = res { error!(error = %e, "Kritik HTTP sunucu hatası."); }
            },
            _ = signal::ctrl_c() => {
                warn!("Kapatma sinyali (Ctrl+C) alındı.");
            }
        }
        
        let _ = http_shutdown_tx.send(());
        cleanup_task.abort();
        info!("✅ Servis başarıyla kapatıldı.");
        Ok(())
    }
}