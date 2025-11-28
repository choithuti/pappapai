// src/core/wallet.rs
use bip39::Mnemonic; // Bỏ Language vì không cần dùng nữa
use rand::rngs::OsRng;
use rand::RngCore;
use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use sha2::{Sha512, Digest};

#[derive(Clone)]
pub struct Wallet {
    pub address: String,
    pub mnemonic: String,
    pub signing_key: SigningKey, 
}

impl Wallet {
    pub fn new() -> Self {
        // 1. Tạo entropy
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);

        // FIX LỖI 1: Bỏ tham số Language::English, chỉ truyền entropy
        let mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to create mnemonic");
        
        // FIX LỖI 2: Collect iterator thành Vec<&str> trước khi join
        let phrase = mnemonic.words().collect::<Vec<&str>>().join(" ");
        
        // Tạo seed
        let seed = mnemonic.to_seed(""); 
        
        // Tạo Key
        let mut hasher = Sha512::new();
        hasher.update(&seed);
        let hash = hasher.finalize();
        
        let mut secret_bytes = [0u8; 32];
        secret_bytes.copy_from_slice(&hash[..32]);
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        
        // Format address
        let pub_bytes = verifying_key.to_bytes();
        let hex_address: String = pub_bytes[..8].iter().map(|b| format!("{:02x}", b)).collect();
        let address = format!("PAPPAP_{}", hex_address);
        
        Self { 
            address, 
            mnemonic: phrase, 
            signing_key 
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.signing_key.sign(msg).to_bytes().to_vec()
    }
}