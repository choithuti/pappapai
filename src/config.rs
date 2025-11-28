// src/config.rs
use config::{Config, File, Environment};
use serde::Deserialize;
use zeroize::Zeroize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub listen_addr: String,
    pub p2p_port: u16,
    pub node_name: String,
    #[serde(skip_deserializing)] // Xử lý riêng để bảo mật
    pub encryption_key: [u8; 32],
}

impl AppConfig {
    pub fn load() -> Self {
        let s = Config::builder()
            // 1. Load từ file config.toml (nếu có)
            .add_source(File::with_name("config").required(false))
            // 2. Load từ biến môi trường (PAPPAP_LISTEN_ADDR, v.v.)
            .add_source(Environment::with_prefix("PAPPAP"))
            .build()
            .expect("Failed to load configuration");

        // Xử lý Key an toàn
        let key_str = s.get::<String>("security.encryption_key")
            .unwrap_or_else(|_| "default_insecure_key_for_dev_32b".to_string());
        
        let mut key_bytes = [0u8; 32];
        let bytes = key_str.as_bytes();
        // Copy tối đa 32 bytes
        let len = bytes.len().min(32);
        key_bytes[..len].copy_from_slice(&bytes[..len]);

        // Zeroize chuỗi string tạm thời để xóa khỏi bộ nhớ
        let mut temp_key = key_str;
        temp_key.zeroize();

        Self {
            listen_addr: s.get::<String>("server.listen_addr").unwrap_or("0.0.0.0:8081".into()),
            p2p_port: s.get::<u16>("network.p2p_port").unwrap_or(9001),
            node_name: s.get::<String>("server.node_name").unwrap_or("Unknown Node".into()),
            encryption_key: key_bytes,
        }
    }
}