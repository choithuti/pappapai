use pqcrypto_dilithium::dilithium5::{
    keypair, 
    detached_sign, 
    verify_detached_signature,
    PublicKey as DilithiumPublicKey, 
    SecretKey as DilithiumSecretKey, 
    DetachedSignature as DilithiumDetachedSignature 
};
// QUAN TRỌNG: Phải import Trait DetachedSignature để dùng .as_bytes() và .from_bytes()
use pqcrypto_traits::sign::{PublicKey, SecretKey, DetachedSignature};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct QuantumWallet {
    pub public_key: Vec<u8>,
    secret_key: Arc<RwLock<Vec<u8>>>,
}

impl QuantumWallet {
    pub fn new() -> Self {
        println!("🛡️  INITIATING QUANTUM SHIELD (Dilithium5)...");
        let (pk, sk) = keypair();
        
        println!("⚛️  QUANTUM KEYS GENERATED | Size: PK={}B, SK={}B", 
            pk.as_bytes().len(), sk.as_bytes().len());

        Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: Arc::new(RwLock::new(sk.as_bytes().to_vec())),
        }
    }

    pub async fn sign_data(&self, data: &[u8]) -> Vec<u8> {
        let sk_bytes = self.secret_key.read().await;
        // Khôi phục Secret Key từ bytes
        let sk = DilithiumSecretKey::from_bytes(&sk_bytes).expect("Invalid Secret Key");
        
        // Tạo chữ ký rời (Detached Signature)
        let sig = detached_sign(data, &sk);
        
        // Chuyển sang bytes (Trait DetachedSignature đã được import nên hàm này hoạt động)
        sig.as_bytes().to_vec()
    }

    pub fn verify_data(data: &[u8], signature: &[u8], pub_key_bytes: &[u8]) -> bool {
        // Khôi phục Public Key
        if let Ok(pk) = DilithiumPublicKey::from_bytes(pub_key_bytes) {
            // Khôi phục Chữ ký (Sửa Signature -> DetachedSignature)
            if let Ok(sig) = DilithiumDetachedSignature::from_bytes(signature) {
                // Xác minh
                return verify_detached_signature(&sig, data, &pk).is_ok();
            }
        }
        false
    }
}
