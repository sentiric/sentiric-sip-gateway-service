// File: src/config.rs
use crate::error::GatewayError;
use std::env;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub struct AppConfig {
    pub listen_addr: SocketAddr,
    pub target_addr: SocketAddr,
    pub public_ip: IpAddr,
    pub public_port: u16,
    pub env: String,
    pub service_version: String,
    pub git_commit: String,
    pub build_date: String,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self, GatewayError> {
        dotenvy::dotenv().ok();

        let env = env::var("ENV").unwrap_or_else(|_| "production".to_string());
        let listen_port_str = env::var("SIP_GATEWAY_LISTEN_PORT").unwrap_or_else(|_| "5060".to_string());
        let listen_port = listen_port_str.parse::<u16>().map_err(|_| GatewayError::ConfigError("Geçersiz SIP_GATEWAY_LISTEN_PORT".to_string()))?;
        
        let target_host = env::var("SIP_SIGNALING_SERVICE_HOST").map_err(|_| GatewayError::ConfigError("ZORUNLU: SIP_SIGNALING_SERVICE_HOST eksik".to_string()))?;
        let target_port = env::var("SIP_SIGNALING_SERVICE_PORT").map_err(|_| GatewayError::ConfigError("ZORUNLU: SIP_SIGNALING_SERVICE_PORT eksik".to_string()))?;
        let target_addr_str = format!("{}:{}", target_host, target_port);
        let target_addr = target_addr_str.parse::<SocketAddr>().map_err(|_| GatewayError::ConfigError(format!("Geçersiz hedef adresi: {}", target_addr_str)))?;

        let public_ip_str = env::var("PUBLIC_IP").map_err(|_| GatewayError::ConfigError("ZORUNLU: PUBLIC_IP (gateway'in genel IP'si) eksik".to_string()))?;
        let public_ip = public_ip_str.parse::<IpAddr>().map_err(|_| GatewayError::ConfigError(format!("Geçersiz PUBLIC_IP adresi: {}", public_ip_str)))?;

        let listen_addr_str = format!("0.0.0.0:{}", listen_port);
        let listen_addr = listen_addr_str.parse::<SocketAddr>().unwrap();

        let service_version = env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
        let git_commit = env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
        let build_date = env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string());

        Ok(AppConfig {
            listen_addr,
            target_addr,
            public_ip,
            public_port: listen_port,
            env,
            service_version,
            git_commit,
            build_date,
        })
    }
}