// src/storage.rs
use sled::Db; // Bỏ IVec
use crate::block::Block;
// Bỏ use serde::{...} thừa nếu không dùng trực tiếp
use std::str;

#[derive(Clone)]
pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).expect("Không thể mở Database");
        println!("💾 STORAGE ENGINE: Sled DB Loaded at '{}'", path);
        Self { db }
    }

    pub fn save_block(&self, block: &Block) {
        let key = format!("block:{}", block.index);
        let encoded: Vec<u8> = bincode::serialize(block).unwrap();
        self.db.insert(key.as_bytes(), encoded).unwrap();
        self.db.insert("chain:height", &block.index.to_be_bytes()).unwrap();
        self.db.insert("chain:last_hash", block.hash.as_bytes()).unwrap();
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