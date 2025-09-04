// File: src/error.rs
use thiserror::Error;
use std::net::SocketAddr;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("UDP soketi '{addr}' adresine bağlanamadı: {source}")]
    SocketBindError { addr: SocketAddr, source: std::io::Error },

    #[error("Yapılandırma hatası: {0}")]
    ConfigError(String),

    #[error("UDP soketinden okuma hatası: {0}")]
    SocketReadError(#[from] std::io::Error),
    
    // Bu varyantı şimdilik kullanmıyoruz ama ileride lazım olabilir.
    // Derleyicinin uyarı vermemesi için `dead_code`'a izin veriyoruz.
    #[allow(dead_code)]
    #[error("Geçersiz UTF-8 SIP paketi alındı")]
    InvalidUtf8,
}