// sentiric-sip-gateway-service/src/config.rs
use crate::error::GatewayError;
use anyhow::{Context, Result};
use std::env;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub struct AppConfig {
    pub listen_addr: SocketAddr,
    pub http_port: u16
    pub target_addr: String,
    pub public_ip: IpAddr,
    pub public_port: u16,
    pub env: String,
    pub service_version: String,
    pub git_commit: String,
    pub build_date: String,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self> {
        
        let listen_port_str = env::var("SIP_GATEWAY_UDP_PORT").unwrap_or_else(|_| "5060".to_string());
        let listen_port = listen_port_str.parse::<u16>()?;

        // --- YENİ SATIRLAR ---
        let http_port_str = env::var("SIP_GATEWAY_HTTP_PORT").unwrap_or_else(|_| "13010".to_string());
        let http_port = http_port_str.parse::<u16>()?;
        // --- BİTİŞ ---

        let target_addr = env::var("SIP_SIGNALING_TARGET_UDP_URL")?;
        let public_ip_str = env::var("SIP_GATEWAY_PUBLIC_IP")?;
        let public_ip = public_ip_str.parse::<IpAddr>()?;
        
        
        let listen_addr_str = format!("0.0.0.0:{}", listen_port);
        let listen_addr = listen_addr_str.parse::<SocketAddr>().unwrap();

        let service_version = env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
        let git_commit = env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
        let build_date = env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string());

        Ok(AppConfig {
            listen_addr,
            http_port, // YENİ SATIR
            target_addr,
            public_ip,
            public_port: listen_port,
            env: env::var("ENV").unwrap_or_else(|_| "production".to_string()),
            service_version,
            git_commit,
            build_date,
        })
    }
}