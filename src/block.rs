use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use crate::transaction::Transaction; // Import Transaction

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub prev_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>, // S?a t? Vec<String> thành Vec<Transaction>
    pub spike_score: f32,
    pub miner: String,
    pub miner_pqc_pubkey: String,
    pub pqc_signature: String,
}

impl Block {
    pub fn new(
        index: u64, 
        prev_hash: String, 
        transactions: Vec<Transaction>, 
        spike_score: f32, 
        miner: String,
        miner_pqc_pubkey: String 
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Self {
            index,
            timestamp,
            prev_hash,
            hash: String::new(),
            transactions,
            spike_score,
            miner,
            miner_pqc_pubkey,
            pqc_signature: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        // Hash danh sách transaction ID
        let tx_data = self.transactions.iter().map(|t| t.id.clone()).collect::<String>();
        
        let input = format!("{}{}{}{}{}{}{}", 
            self.index, 
            self.timestamp, 
            self.prev_hash, 
            self.spike_score, 
            self.miner,
            self.miner_pqc_pubkey,
            tx_data
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}
