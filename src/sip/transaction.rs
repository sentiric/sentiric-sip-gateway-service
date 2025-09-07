// File: src/sip/transaction.rs

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, debug};

#[derive(Clone, Debug)]
pub struct TransactionInfo {
    pub original_client_addr: SocketAddr,
    pub original_via_headers: Vec<String>, // 'Via' başlıklarının tamamını saklar.
    #[allow(dead_code)] // Bu alan giden BYE/CANCEL istekleri için saklanıyor.
    pub original_contact_header: String,
    #[allow(dead_code)] // Bu alan giden BYE/CANCEL istekleri için saklanıyor.
    pub record_route_header: Option<String>,
    pub created_at: Instant,
}

pub type TransactionKey = (String, String); // (Call-ID, CSeq Method)
pub type Transactions = Arc<Mutex<HashMap<TransactionKey, TransactionInfo>>>;

pub fn new_transaction_manager() -> Transactions {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn cleanup_old_transactions(transactions: Transactions) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut guard = transactions.lock().await;
        let before_count = guard.len();
        guard.retain(|_key, tx_info| tx_info.created_at.elapsed() < Duration::from_secs(120));
        let after_count = guard.len();
        if before_count > after_count {
            info!(
                cleaned_count = before_count - after_count,
                remaining_count = after_count,
                "Temizlik görevi: Eski işlemler temizlendi."
            );
        } else if before_count > 0 {
             debug!(remaining_count = after_count, "Temizlik görevi çalıştı, eski işlem bulunamadı.");
        }
    }
}