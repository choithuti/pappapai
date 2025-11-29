use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::snn_core::SNNCore;
use rand::Rng;

pub struct AutoTrainer;

impl AutoTrainer {
    pub async fn start(snn: Arc<SNNCore>) {
        println!("🏋️  AUTO-TRAINER: Started background learning loop...");
        
        loop {
            // 1. Nghỉ ngơi giữa các hiệp
            sleep(Duration::from_secs(5)).await;

            // FIX LỖI SEND: Đóng gói việc tạo RNG trong block {}
            // Biến rng sẽ được tạo ra và HỦY ngay lập tức sau khi tính xong 'noise'
            let noise = {
                let mut rng = rand::thread_rng();
                rng.gen_range(0.5..1.5)
            }; 
            
            // 2. Thực hiện huấn luyện (Lúc này rng đã chết, await an toàn)
            let adaptation = snn.train_step(noise).await;

            // 3. Log kết quả
            if adaptation > 0.5 {
                println!("🧠 Brain plasticity updated. Adaptation index: {:.4}", adaptation);
            }
        }
    }
}
