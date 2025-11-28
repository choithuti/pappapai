use tokio::sync::RwLock;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use sysinfo::{System, SystemExt};
use std::sync::Arc;
use crate::storage::Storage; // Import Storage

#[derive(Clone, Debug)]
pub struct BioNeuron {
    pub potential: f32,
    pub threshold: f32,
    pub decay: f32,
    pub refractory_timer: u8,
}

pub struct SNNCore {
    neurons: RwLock<Vec<BioNeuron>>,
    weights: Vec<f32>,
    storage: Arc<Storage>, // Thay HashMap bằng Storage
    total_neurons: usize,
}

impl SNNCore {
    pub fn new(storage: Arc<Storage>) -> Self {
        // Auto-scale logic (giữ nguyên)
        let mut sys = System::new_all();
        sys.refresh_all();
        let total_memory_kb = sys.total_memory();
        let scale_factor = 5_000; // Giảm nhẹ để dành RAM cho DB
        let ram_gb = total_memory_kb / (1024 * 1024);
        let neuron_count = (ram_gb as usize * scale_factor).max(1024);

        println!("🧠 BIO-SNN LOADED | Neurons: {} | Storage: PERSISTENT", neuron_count);

        let mut rng = rand::thread_rng();
        let mut neurons = Vec::with_capacity(neuron_count);
        let mut weights = Vec::with_capacity(neuron_count);

        for _ in 0..neuron_count {
            neurons.push(BioNeuron {
                potential: -70.0,
                threshold: -55.0 + rng.gen_range(-5.0..5.0),
                decay: 0.95,
                refractory_timer: 0,
            });
            weights.push(rng.gen_range(0.1..0.5));
        }

        Self {
            neurons: RwLock::new(neurons),
            weights,
            storage,
            total_neurons: neuron_count,
        }
    }

    // Mining Forward (Giữ nguyên logic cũ)
    pub async fn forward(&self, _input: f32) -> f32 {
        let mut neurons = self.neurons.write().await;
        let mut active_count = 0.0;
        let mut rng = rand::thread_rng();
        let sample = 1024.min(self.total_neurons);

        for i in 0..sample {
            let n = &mut neurons[i];
            if n.refractory_timer > 0 { n.refractory_timer -= 1; continue; }
            n.potential += rng.gen_range(2.0..10.0);
            if n.potential >= n.threshold {
                n.potential = -85.0;
                n.refractory_timer = 5;
                active_count += 1.0;
            } else {
                n.potential *= n.decay;
            }
        }
        1.0 + (active_count / sample as f32)
    }

    pub async fn stats(&self) -> (usize, f32) {
        (self.total_neurons, self.total_neurons as f32 * 0.01)
    }

    // Process Text & Recall Memory from DISK
    pub async fn process_text(&self, text: &str) -> (f32, String, String) {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(seed);
        
        let intensity = rng.gen_range(0.0..100.0);
        let score = 1.0 + (intensity / 100.0);
        let mood = if score < 1.3 { "😴 Calm" } else { "🔥 Excited" };

        // TRUY HỒI TỪ Ổ CỨNG
        let reply = self.storage.recall_fact(text)
            .unwrap_or_else(|| "Tôi chưa biết. Dạy tôi bằng lệnh /teach".to_string());

        (score, mood.to_string(), reply)
    }

    // Lưu kiến thức vào Ổ CỨNG (Vĩnh viễn)
    pub async fn learn(&self, key: String, value: String) {
        self.storage.learn_fact(&key, &value);
        // Có thể thêm logic tăng trọng số nơ-ron ở đây để mô phỏng Long-term Potentiation (LTP)
    }
}
