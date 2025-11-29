use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: i64,
    pub signature: String,
}

impl Transaction {
    pub fn calculate_hash(&self) -> String {
        let payload = format!(
            "{}:{}:{}:{}:{}:{}",
            self.sender, self.receiver, self.amount, self.fee, self.nonce, self.timestamp
        );
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }

    pub fn verify(&self) -> bool {
        if let Ok(pub_bytes) = hex::decode(&self.sender) {
            if let Ok(pub_key) = VerifyingKey::from_bytes(pub_bytes.as_slice().try_into().unwrap()) {
                if let Ok(sig_bytes) = hex::decode(&self.signature) {
                    if sig_bytes.len() == 64 {
                        let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
                        let signature = Signature::from_bytes(&sig_arr);
                        
                        let payload = format!(
                            "{}:{}:{}:{}:{}:{}",
                            self.sender, self.receiver, self.amount, self.fee, self.nonce, self.timestamp
                        );
                        return pub_key.verify(payload.as_bytes(), &signature).is_ok();
                    }
                }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct Mempool {
    pub pending: Arc<RwLock<HashMap<String, Transaction>>>,
}

impl Mempool {
    pub fn new() -> Self {
        Self { pending: Arc::new(RwLock::new(HashMap::new())) }
    }
    pub fn add_tx(&self, tx: Transaction) -> bool {
        let mut pool = self.pending.write().unwrap();
        if pool.contains_key(&tx.id) { return false; }
        pool.insert(tx.id.clone(), tx);
        true
    }
    pub fn pop_n(&self, n: usize) -> Vec<Transaction> {
        let mut pool = self.pending.write().unwrap();
        let keys: Vec<String> = pool.keys().take(n).cloned().collect();
        let mut txs = Vec::new();
        for key in keys { if let Some(tx) = pool.remove(&key) { txs.push(tx); } }
        txs
    }
}
