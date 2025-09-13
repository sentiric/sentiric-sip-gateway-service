// File: src/network.rs
use crate::config::AppConfig;
use crate::error::GatewayError;
use crate::sip::handler;
use crate::sip::transaction::Transactions;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, warn};

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
        let (len, remote_addr) = match sock.recv_from(&mut buf).await {
            Ok(result) => result,
            Err(e) => {
                // =========================================================================
                //   SON LOG İYİLEŞTİRMESİ BURADA
                // =========================================================================
                if e.kind() == ErrorKind::ConnectionReset {
                    warn!(
                        error_kind = ?e.kind(),
                        "Ağ dinleme hatası (ConnectionReset): Bu durum genellikle ulaşılamayan bir hedefe (örn: kapalı sip-signaling) paket gönderildikten sonra oluşur. Dinleyici devam ediyor."
                    );
                    continue; 
                }
                // =========================================================================
                
                error!(error = %e, "Soketten okuma sırasında kritik bir hata oluştu. Servis durdurulacak.");
                return Err(e.into());
            }
        };
        
        let packet_str = match std::str::from_utf8(&buf[..len]) {
            Ok(s) => s.to_string(),
            Err(_) => {
                warn!(source = %remote_addr, "UTF-8 olmayan bir paket alındı, atlanıyor.");
                continue; 
            }
        };

        let sock_clone = Arc::clone(&sock);
        let transactions_clone = transactions.clone();
        let config_clone = Arc::clone(&config);
        
        tokio::spawn(async move {
            handler::handle_packet(&packet_str, remote_addr, &sock_clone, &transactions_clone, &config_clone).await;
        });
    }
}