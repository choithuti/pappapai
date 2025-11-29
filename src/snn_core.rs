use tokio::sync::RwLock;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use sysinfo::{System, SystemExt};
use std::sync::Arc;
use crate::storage::Storage;
use crate::oracle::Oracle;
use crate::llm::LLMBridge;
// Đã xóa 'use regex::Regex' ở đây

#[derive(Clone, Debug)]
pub struct BioNeuron {
    pub potential: f32, pub threshold: f32, pub decay: f32,
    pub refractory_timer: u8, pub sensitivity: f32, 
}

pub struct SNNCore {
    neurons: RwLock<Vec<BioNeuron>>,
    storage: Arc<Storage>,
    oracle: Oracle,
    llm: LLMBridge,
    total_neurons: usize,
}

impl SNNCore {
    pub fn new(storage: Arc<Storage>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let ram_gb = sys.total_memory() / (1024 * 1024);
        let neuron_count = ((ram_gb as usize * 50_000).max(10_000)).min(1_000_000);
        println!("🧠 PẬP PẬP INTELLIGENCE ONLINE | Neurons: {}", neuron_count);

        let mut rng = rand::thread_rng();
        let mut neurons = Vec::with_capacity(neuron_count);
        for _ in 0..neuron_count {
            neurons.push(BioNeuron { potential: -70.0, threshold: -55.0, decay: 0.95, refractory_timer: 0, sensitivity: 1.0 });
        }

        Self {
            neurons: RwLock::new(neurons),
            storage,
            oracle: Oracle::new(),
            llm: LLMBridge::new(),
            total_neurons: neuron_count,
        }
    }

    pub async fn train_step(&self, intensity: f32) -> f32 {
        let mut neurons = self.neurons.write().await;
        let sample = 1024.min(self.total_neurons);
        let mut active = 0.0;
        let mut rng = rand::thread_rng(); // FIX: Khai báo rng ở đây để dùng trong vòng lặp

        for i in 0..sample {
            let n = &mut neurons[i];
            n.potential += intensity * n.sensitivity + rng.gen_range(0.0..1.0); // Thêm nhiễu ngẫu nhiên
            if n.potential >= n.threshold { n.potential = -85.0; active += 1.0; n.sensitivity = (n.sensitivity + 0.01).min(3.0); }
            else { n.potential *= n.decay; }
        }
        active / sample as f32
    }

    pub async fn forward(&self, _input: f32) -> f32 { self.train_step(1.0).await }
    pub async fn stats(&self) -> (usize, f32) { (self.total_neurons, 1024.0) }
    pub async fn learn(&self, k: String, v: String) { self.storage.learn_fact(&k, &v); }

    pub async fn process_text(&self, text: &str) -> (f32, String, String) {
        let mut hasher = DefaultHasher::new(); text.hash(&mut hasher);
        let mut rng = StdRng::seed_from_u64(hasher.finish());
        let score = 1.0 + rng.gen_range(0.0..1.5);
        let mood = if score < 1.5 { "⚡ Nhanh" } else { "🧠 Sâu" };

        if let Some(ans) = self.storage.recall_fact(text) { return (score, mood.to_string(), ans); }

        let mut final_answer = String::new();

        if let Ok(res) = self.oracle.smart_search(text).await {
            if !res.contains("Không tìm thấy") && res.len() > 20 {
                final_answer = res;
            }
        }

        if final_answer.is_empty() {
            println!("⚠️ Web failed. Calling LLM...");
            match self.llm.ask_ai(text).await {
                Ok(ai_res) => {
                    final_answer = ai_res;
                },
                Err(_) => {
                    final_answer = "Hiện tại Pập Pập chưa tìm thấy thông tin này trên Internet.".to_string();
                }
            }
        }

        if !final_answer.contains("chưa tìm thấy") {
            self.storage.learn_fact(text, &final_answer);
        }
        
        (score, mood.to_string(), final_answer)
    }
}
