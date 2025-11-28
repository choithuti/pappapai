use crate::{snn_core::SNNCore, block::Block, storage::Storage, quantum::QuantumWallet};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;
use std::io::{self, Write};

pub struct PappapChain {
    pub snn: Arc<SNNCore>,
    pub height: Arc<RwLock<u64>>,
    pub last_hash: Arc<RwLock<String>>,
    pub storage: Arc<Storage>,
    pub blocks_history: Arc<RwLock<VecDeque<Block>>>,
    pub wallet: Arc<QuantumWallet>, // Ví lượng tử tích hợp
}

impl PappapChain {
    pub async fn new() -> Self {
        let storage = Arc::new(Storage::new("pappap_data"));
        let saved_height = storage.get_height();
        let saved_hash = storage.get_last_hash();
        
        // Khởi tạo Ví Lượng Tử
        let wallet = Arc::new(QuantumWallet::new());

        println!("🔄 CHAIN RESUMED | Height: {} | PQC Wallet Ready", saved_height);

        Self {
            snn: Arc::new(SNNCore::new(storage.clone())),
            height: Arc::new(RwLock::new(saved_height)),
            last_hash: Arc::new(RwLock::new(saved_hash)),
            blocks_history: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            storage,
            wallet,
        }
    }

    pub async fn run(&self) {
        println!("⛏️  MINING (PQC SECURED) STARTED...");
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(800));
        
        // Lấy Public Key dạng Hex để gắn vào block
        let pub_key_hex = hex::encode(&self.wallet.public_key);

        loop {
            interval.tick().await;

            // 1. Update Height
            let mut h_guard = self.height.write().await;
            *h_guard += 1;
            let current_height = *h_guard;
            drop(h_guard);

            let prev_hash = self.last_hash.read().await.clone();
            
            // 2. AI Mining
            let spike = self.snn.forward(1.0).await;

            // 3. Tạo Block (Chưa ký)
            let mut block = Block::new(
                current_height,
                prev_hash,
                vec![],
                spike,
                "choithuti_PQC_NODE".to_string(),
                pub_key_hex.clone(),
            );

            // 4. KÝ SỐ BẢO MẬT LƯỢNG TỬ (Dilithium Signing)
            // Ký lên Hash của block
            let signature_bytes = self.wallet.sign_data(block.hash.as_bytes()).await;
            block.pqc_signature = hex::encode(signature_bytes);

            // 5. Lưu trữ
            let mut last_hash_guard = self.last_hash.write().await;
            *last_hash_guard = block.hash.clone();
            
            self.storage.save_block(&block);
            
            // Cập nhật RAM history
            let mut history = self.blocks_history.write().await;
            if history.len() >= 15 { history.pop_front(); }
            history.push_back(block.clone());

            print!("\r\x1b[K🛡️  Block #{} | PQC Sig: {}... | Spike: {:.4}", 
                block.index, &block.pqc_signature[..8], block.spike_score);
            io::stdout().flush().unwrap();
        }
    }
}
