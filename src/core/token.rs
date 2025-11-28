// src/core/token.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PappapToken {
    pub name: String,
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
    pub balances: HashMap<String, u128>,
}

impl PappapToken {
    pub fn genesis() -> Self {
        let mut balances = HashMap::new();
        // Phân bổ mẫu (Đơn vị: Wei/Satoshi - nhỏ nhất)
        balances.insert("GENESIS_WALLET_VN".to_string(), 300_000_000_000_000_000_000); 
        balances.insert("COMMUNITY_POOL".to_string(),    700_000_000_000_000_000_000);

        Self {
            name: "Pappap AI Token".to_string(),
            symbol: "PAPPAP".to_string(),
            total_supply: 1_000_000_000_000_000_000_000, // 1 Tỷ token
            decimals: 18,
            balances,
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u128) -> Result<(), &'static str> {
        let sender_bal = self.balances.get(from).copied().unwrap_or(0);
        if sender_bal < amount {
            return Err("Insufficient balance");
        }

        self.balances.insert(from.to_string(), sender_bal - amount);
        let receiver_bal = self.balances.get(to).copied().unwrap_or(0);
        self.balances.insert(to.to_string(), receiver_bal + amount);
        
        Ok(())
    }
}