use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub prev_hash: String,
    pub hash: String,
    pub transactions: Vec<String>,
    pub spike_score: f32,
    pub miner: String,
    // --- CÁC TRƯỜNG MỚI CHO PQC ---
    pub miner_pqc_pubkey: String, // Hex string
    pub pqc_signature: String,    // Hex string
}

impl Block {
    pub fn new(
        index: u64, 
        prev_hash: String, 
        transactions: Vec<String>, 
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
            pqc_signature: String::new(), // Sẽ ký sau
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let tx_data = self.transactions.join("");
        let input = format!("{}{}{}{}{}{}{}", 
            self.index, 
            self.timestamp, 
            self.prev_hash, 
            self.spike_score, 
            self.miner,
            self.miner_pqc_pubkey, // Hash cả Public Key lượng tử
            tx_data
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}
