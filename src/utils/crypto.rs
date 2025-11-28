// src/utils/crypto.rs
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
use rand::RngCore;

pub struct CryptoEngine {
    cipher: Aes256Gcm,
}

impl CryptoEngine {
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        Self { cipher: Aes256Gcm::new(key) }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        match self.cipher.encrypt(nonce, data) {
            Ok(ciphertext) => [nonce_bytes.to_vec(), ciphertext].concat(),
            Err(_) => vec![], // Nên handle error tốt hơn trong production
        }
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        if data.len() < 12 { return Err("Invalid data length"); }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed")
    }
}