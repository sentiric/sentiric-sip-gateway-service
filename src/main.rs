// sentiric-sip-gateway-service/src/main.rs
use anyhow::Result;
// DÜZELTME: Kendi crate'imiz içindeki bir modüle erişmek için 'crate::' kullanılır.
use crate::app::App;

// DÜZELTME: Derleyicinin projedeki diğer modülleri bulabilmesi için
// ana dosyada (main.rs) bildirimleri yapılmalıdır.
mod app;
mod config;
mod error;
mod network;
mod sip;

#[tokio::main]
async fn main() -> Result<()> {
    App::bootstrap().await?.run().await
}