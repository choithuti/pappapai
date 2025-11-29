use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct WebWorker {
    pub last_seen: u64,
    pub hashrate: f32,
}

pub struct WebNodeManager {
    workers: RwLock<HashMap<String, WebWorker>>,
}

impl WebNodeManager {
    pub fn new() -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_beat(&self, client_id: String, hashrate: f32) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut w = self.workers.write().await;
        w.insert(client_id, WebWorker { last_seen: now, hashrate });
    }

    pub async fn get_stats(&self) -> (usize, f32) {
        let w = self.workers.read().await;
        let count = w.len();
        let total_power: f32 = w.values().map(|v| v.hashrate).sum();
        (count, total_power)
    }

    pub async fn prune_offline(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut w = self.workers.write().await;
        w.retain(|_, v| now - v.last_seen < 15);
    }
}
