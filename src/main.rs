// sentiric-sip-gateway-service/src/main.rs
use anyhow::Result;
use sentiric_sip_gateway_service::app::App;

mod app;
mod config;
mod error;
mod network;
mod sip;

#[tokio::main]
async fn main() -> Result<()> {
    // Uygulamanın başlatılması ve çalıştırılması sorumluluğu
    // tamamen 'app' modülüne devredildi.
    App::bootstrap().await?.run().await
}