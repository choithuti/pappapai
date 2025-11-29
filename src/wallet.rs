use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Sha256, Digest};
use bip39::{Mnemonic, Language};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: String,
    pub address: String,
    pub mnemonic: String,
    #[serde(skip_serializing)] pub secret_key: Vec<u8>,
}

impl Wallet {
    pub fn new() -> Self {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
        let phrase = mnemonic.words().collect::<Vec<&str>>().join(" ");
        Self::create(mnemonic, phrase)
    }
    pub fn recover(phrase: &str) -> Result<Self, &'static str> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase).map_err(|_| "Invalid Mnemonic")?;
        Ok(Self::create(mnemonic, phrase.to_string()))
    }
    fn create(mnemonic: Mnemonic, phrase: String) -> Self {
        let seed = mnemonic.to_seed("");
        let signing_key = SigningKey::from_bytes(&seed[0..32].try_into().unwrap());
        let verifying_key = VerifyingKey::from(&signing_key);
        let pub_hex = hex::encode(verifying_key.to_bytes());
        let mut hasher = Sha256::new(); hasher.update(verifying_key.to_bytes());
        let address = format!("PAPPAP{}", hex::encode(&hasher.finalize()[0..16])).to_uppercase();
        Self { public_key: pub_hex, address, mnemonic: phrase, secret_key: signing_key.to_bytes().to_vec() }
    }
    pub fn sign(&self, message: &[u8]) -> String {
        let sk = SigningKey::from_bytes(self.secret_key.as_slice().try_into().unwrap());
        let sig: Signature = sk.sign(message);
        hex::encode(sig.to_bytes())
    }
}
