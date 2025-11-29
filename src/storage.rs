use sled::Db;
use crate::block::Block;
use serde::{Serialize, Deserialize};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeStats {
    pub first_seen: u64,      // Ngày tạo Node
    pub total_starts: u64,    // Số lần khởi động
    pub total_blocks: u64,    // Tổng block đã đào
    pub reputation: u64,      // Điểm uy tín
}

#[derive(Clone)]
pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).expect("Không thể mở Database");
        println!("💾 STORAGE: Connected to '{}'", path);
        Self { db }
    }

    // --- QUẢN LÝ IDENTITY (KHÓA BÍ MẬT) ---
    pub fn save_node_secret(&self, secret_bytes: &[u8]) {
        self.db.insert("node:secret_key", secret_bytes).unwrap();
    }

    pub fn load_node_secret(&self) -> Option<Vec<u8>> {
        self.db.get("node:secret_key").unwrap().map(|ivec| ivec.to_vec())
    }

    // --- QUẢN LÝ THỐNG KÊ (STATS) ---
    pub fn load_stats(&self) -> NodeStats {
        match self.db.get("node:stats").unwrap() {
            Some(ivec) => bincode::deserialize(&ivec).unwrap_or(NodeStats {
                first_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                total_starts: 0,
                total_blocks: 0,
                reputation: 100,
            }),
            None => NodeStats {
                first_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                total_starts: 0,
                total_blocks: 0,
                reputation: 100,
            }
        }
    }

    pub fn save_stats(&self, stats: &NodeStats) {
        let encoded = bincode::serialize(stats).unwrap();
        self.db.insert("node:stats", encoded).unwrap();
    }

    // --- QUẢN LÝ BLOCKCHAIN (Cũ) ---
    pub fn save_block(&self, block: &Block) {
        let key = format!("block:{}", block.index);
        let encoded: Vec<u8> = bincode::serialize(block).unwrap();
        self.db.insert(key.as_bytes(), encoded).unwrap();
        self.db.insert("chain:height", &block.index.to_be_bytes()).unwrap();
        self.db.insert("chain:last_hash", block.hash.as_bytes()).unwrap();
        
        // Tự động tăng stats khi lưu block
        let mut stats = self.load_stats();
        stats.total_blocks += 1;
        self.save_stats(&stats);
    }

    pub fn get_height(&self) -> u64 {
        match self.db.get("chain:height").unwrap() {
            Some(ivec) => {
                let bytes: [u8; 8] = ivec.as_ref().try_into().unwrap();
                u64::from_be_bytes(bytes)
            },
            None => 0
        }
    }

    pub fn get_last_hash(&self) -> String {
        match self.db.get("chain:last_hash").unwrap() {
            Some(ivec) => str::from_utf8(&ivec).unwrap().to_string(),
            None => "pappap-genesis-vn-2025".to_string()
        }
    }

    pub fn get_recent_blocks(&self, limit: u64) -> Vec<Block> {
        let height = self.get_height();
        let start = height.saturating_sub(limit);
        let mut blocks = Vec::new();
        for i in (start..=height).rev() {
            let key = format!("block:{}", i);
            if let Ok(Some(ivec)) = self.db.get(key.as_bytes()) {
                if let Ok(block) = bincode::deserialize::<Block>(&ivec) {
                    blocks.push(block);
                }
            }
        }
        blocks
    }

    // --- AI MEMORY ---
    pub fn learn_fact(&self, question: &str, answer: &str) {
        let key = format!("ai:mem:{}", question.to_lowercase());
        self.db.insert(key.as_bytes(), answer.as_bytes()).unwrap();
    }

    pub fn recall_fact(&self, question: &str) -> Option<String> {
        let key = format!("ai:mem:{}", question.to_lowercase());
        match self.db.get(key.as_bytes()).unwrap() {
            Some(ivec) => Some(str::from_utf8(&ivec).ok()?.to_string()),
            None => None
        }
    }
}
