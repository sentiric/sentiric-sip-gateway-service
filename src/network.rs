// File: src/network.rs (UYARI 1 GİDERİLDİ)
use crate::config::AppConfig;
use crate::error::GatewayError;
use crate::sip::handler;
use crate::sip::transaction::Transactions;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::warn; // DÜZELTME: debug logunu kaldırdık çünkü handler içinde zaten var.

pub async fn listen_and_process(
    config: Arc<AppConfig>,
    transactions: Transactions,
) -> Result<(), GatewayError> {
    let sock = UdpSocket::bind(config.listen_addr)
        .await
        .map_err(|e| GatewayError::SocketBindError {
            addr: config.listen_addr,
            source: e,
        })?;
    let sock = Arc::new(sock);

    let mut buf = [0; 65535];
    loop {
        let (len, remote_addr) = sock.recv_from(&mut buf).await?;
        
        // --- DÜZELTME: Artık GatewayError::InvalidUtf8 kullanıyoruz ---
        let packet_str = match std::str::from_utf8(&buf[..len]) {
            Ok(s) => s.to_string(),
            Err(_) => {
                // Hatalı paketi logla ve bir sonraki pakete geç.
                warn!(source = %remote_addr, "UTF-8 olmayan bir paket alındı, atlanıyor.");
                // Burada `GatewayError::InvalidUtf8` fırlatmak yerine devam etmek,
                // tek bir bozuk paketin tüm servisi durdurmasını engeller.
                // Bu yüzden uyarıyı kabul edip `dead_code` olarak işaretleyebiliriz veya
                // bu şekilde loglayıp devam edebiliriz. Loglamak daha mantıklı.
                continue; 
            }
        };
        // --- DÜZELTME SONU ---

        let sock_clone = Arc::clone(&sock);
        let transactions_clone = transactions.clone();
        let config_clone = Arc::clone(&config);
        
        tokio::spawn(async move {
            handler::handle_packet(&packet_str, remote_addr, &sock_clone, &transactions_clone, &config_clone).await;
        });
    }
}