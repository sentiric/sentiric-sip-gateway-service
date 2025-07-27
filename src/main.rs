use std::env;
// use std::net::SocketAddr; // <-- Artık kullanılmıyor, siliyoruz.
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{info, error}; // <-- 'Level' ve 'debug' kullanılmadığı için kaldırıldı.
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let listen_port = env::var("LISTEN_PORT").unwrap_or_else(|_| "5060".to_string());
    let target_host = env::var("TARGET_HOST").unwrap_or_else(|_| "sip-signaling".to_string());
    let target_port = env::var("TARGET_PORT").unwrap_or_else(|_| "5060".to_string());

    let listen_addr = format!("0.0.0.0:{}", listen_port);
    let target_addr = format!("{}:{}", target_host, target_port);

    let sock = Arc::new(UdpSocket::bind(&listen_addr).await?);
    
    info!(
        listen_addr = %listen_addr,
        target_addr = %target_addr,
        "✅ SIP Gateway (Rust) proxy başlatıldı."
    );

    let mut buf = [0; 65535];

    loop {
        let (len, client_addr) = sock.recv_from(&mut buf).await?;

        info!(
            source = %client_addr,
            bytes = len,
            "➡️  IN"
        );

        // Hedefe paketi gönder
        if let Err(e) = sock.send_to(&buf[..len], &target_addr).await {
            error!(error = %e, "❌ Paketi hedefe yönlendirme hatası.");
        }
    }
}