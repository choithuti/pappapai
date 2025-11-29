use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

struct CacheItem {
    value: String,
    expires_at: Instant,
}

pub struct SmartCache {
    store: RwLock<HashMap<String, CacheItem>>,
    ttl: Duration,
}

impl SmartCache {
    pub fn new() -> Self {
        println!("⚡ SMART CACHE ACTIVATED (TTL: 1 Hour)");
        Self {
            store: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(3600), // Lưu trong 1 giờ
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        if let Some(item) = store.get(key) {
            if item.expires_at > Instant::now() {
                return Some(item.value.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: String) {
        let mut store = self.store.write().await;
        store.insert(key, CacheItem {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    // Dọn dẹp bộ nhớ định kỳ
    pub async fn prune(&self) {
        let mut store = self.store.write().await;
        let now = Instant::now();
        store.retain(|_, v| v.expires_at > now);
    }
}
