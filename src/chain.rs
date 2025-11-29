use crate::{snn_core::SNNCore, block::Block, storage::Storage, quantum::QuantumWallet, cache::SmartCache, p2p::P2PNode, transaction::Mempool};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::Instant;
use bincode;

pub struct PappapChain {
    pub snn: Arc<SNNCore>,
    pub height: Arc<RwLock<u64>>,
    pub last_hash: Arc<RwLock<String>>,
    pub storage: Arc<Storage>,
    pub blocks_history: Arc<RwLock<VecDeque<Block>>>,
    pub wallet: Arc<QuantumWallet>,
    pub p2p_node: Arc<Mutex<P2PNode>>,
    pub mempool: Arc<Mempool>,
}

impl PappapChain {
    // S?A L?I: Nh?n d? 3 tham s? d? kh?p v?i main.rs
    pub async fn new(storage: Arc<Storage>, cache: SmartCache, p2p_node: Arc<Mutex<P2PNode>>) -> Self {
        let saved_height = storage.get_height();
        let saved_hash = storage.get_last_hash();
        let wallet = Arc::new(QuantumWallet::new());
        
        // S?A L?I: Truy?n d? storage và cache vào SNNCore
        let snn = Arc::new(SNNCore::new(storage.clone(), cache));
        
        let mempool = Arc::new(Mempool::new());

        println!("?? CHAIN SYNCED | Height: {}", saved_height);

        Self {
            snn,
            height: Arc::new(RwLock::new(saved_height)),
            last_hash: Arc::new(RwLock::new(saved_hash)),
            blocks_history: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            storage,
            wallet,
            p2p_node,
            mempool,
        }
    }

    pub async fn run(&self) {
        println!("??  MINING STARTED (800ms/block)");
        let pub_key_hex = hex::encode(&self.wallet.public_key);
        
        loop {
            let start = Instant::now();

            // 1. Mining
            let spike = self.snn.forward(1.0).await;

            // 2. Update Height
            let mut h_guard = self.height.write().await;
            *h_guard += 1;
            let current_height = *h_guard;
            drop(h_guard);

            let prev_hash = self.last_hash.read().await.clone();

            // 3. L?y giao d?ch
            let txs = self.mempool.pop_n(50);
            let tx_count = txs.len();

            // 4. T?o Block
            let mut block = Block::new(
                current_height,
                prev_hash,
                txs,
                spike,
                "choithuti_NODE".to_string(),
                pub_key_hex.clone(),
            );

            // 5. Ký & Luu
            let signature = self.wallet.sign_data(block.hash.as_bytes()).await;
            block.pqc_signature = hex::encode(signature);
            
            *self.last_hash.write().await = block.hash.clone();
            self.storage.save_block(&block);
            
            let mut history = self.blocks_history.write().await;
            if history.len() >= 15 { history.pop_front(); }
            history.push_back(block.clone());
            drop(history);

            // 6. Broadcast P2P
            if let Ok(data) = bincode::serialize(&block) {
                let mut p2p = self.p2p_node.lock().await;
                p2p.broadcast_block(data);
            }

            let elapsed = start.elapsed();
            print!("\r\x1b[K?? Block #{} | Tx: {} | Spike: {:.3}", block.index, tx_count, block.spike_score);
            io::stdout().flush().unwrap();

            if elapsed.as_millis() < 800 {
                tokio::time::sleep(std::time::Duration::from_millis(800) - elapsed).await;
            }
        }
    }
}
