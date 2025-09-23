// sentiric-sip-gateway-service/src/error.rs
use thiserror::Error;
use std::net::{AddrParseError, SocketAddr};
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("UDP soketi '{addr}' adresine bağlanamadı: {source}")]
    SocketBindError { addr: SocketAddr, source: std::io::Error },

    // DÜZELTME: Bu varyant artık kullanılmadığı için kaldırıldı.
    // #[error("Yapılandırma hatası: {0}")]
    // ConfigError(String),

    #[error("UDP soketinden okuma hatası: {0}")]
    SocketReadError(#[from] std::io::Error),

    #[error("Geçersiz IP adresi: {0}")]
    AddrParse(#[from] AddrParseError),

    #[error("Geçersiz port numarası: {0}")]
    PortParse(#[from] ParseIntError),
    
    #[allow(dead_code)]
    #[error("Geçersiz UTF-8 SIP paketi alındı")]
    InvalidUtf8,
}