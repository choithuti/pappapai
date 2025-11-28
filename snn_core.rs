// src/snn_core.rs
use tokio::sync::RwLock;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use sysinfo::{System, SystemExt}; // Thư viện đọc phần cứng

// Mô hình Nơ-ron chuẩn sinh học (LIF)
#[derive(Clone, Debug)]
pub struct BioNeuron {
    pub potential: f32,      // Điện thế màng (Membrane Potential)
    pub threshold: f32,      // Ngưỡng kích hoạt
    pub decay: f32,          // Hệ số rò rỉ (Leak factor)
    pub refractory_timer: u8,// Thời gian trơ (không thể kích hoạt lại ngay)
}

pub struct SNNCore {
    neurons: RwLock<Vec<BioNeuron>>, // Dùng Vec động thay vì mảng tĩnh
    weights: Vec<f32>,               // Synapse weights
    memory: RwLock<HashMap<String, String>>,
    total_neurons: usize,
}

impl SNNCore {
    pub fn new() -> Self {
        // 1. AUTO-SCALE: Kiểm tra phần cứng
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let total_memory_kb = sys.total_memory();
        // Công thức: Cứ 1GB RAM = 10,000 Neurons (Tối ưu để không tràn RAM)
        // Ví dụ: VPS 2GB RAM -> ~20,000 Neurons
        let scale_factor = 10_000; 
        let ram_gb = total_memory_kb / (1024 * 1024);
        let neuron_count = (ram_gb as usize * scale_factor).max(1024); // Tối thiểu 1024

        println!("🖥️  HARDWARE DETECTED: {} GB RAM", ram_gb);
        println!("🧠 BIO-SNN SCALING: Initializing {} Bio-Neurons...", neuron_count);

        let mut rng = rand::thread_rng();
        
        // 2. Khởi tạo Nơ-ron sinh học
        let mut neurons = Vec::with_capacity(neuron_count);
        let mut weights = Vec::with_capacity(neuron_count);

        for _ in 0..neuron_count {
            neurons.push(BioNeuron {
                potential: -70.0, // Điện thế nghỉ (Resting potential)
                threshold: -55.0 + rng.gen_range(-5.0..5.0), // Ngưỡng sinh học (-55mV)
                decay: 0.95, // Rò rỉ 5% mỗi chu kỳ
                refractory_timer: 0,
            });
            weights.push(rng.gen_range(0.1..0.5)); // Trọng số synapse
        }

        Self {
            neurons: RwLock::new(neurons),
            weights,
            memory: RwLock::new(HashMap::new()),
            total_neurons: neuron_count,
        }
    }

    pub async fn stats(&self) -> (usize, f32) {
        (self.total_neurons, self.total_neurons as f32 * 0.01) // Power giả lập
    }

    // Xử lý Text -> Cảm xúc (Mood)
    pub async fn process_text(&self, text: &str) -> (f32, String, String) {
        // 1. Hash text thành Seed kích thích
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(seed);
        
        let mut neurons = self.neurons.write().await;
        let mut active_count = 0.0;

        // 2. Quá trình Lan truyền Xung thần kinh (Spiking Dynamics)
        for (i, neuron) in neurons.iter_mut().enumerate() {
            // A. Refractory Period (Giai đoạn trơ)
            if neuron.refractory_timer > 0 {
                neuron.refractory_timer -= 1;
                // Hồi phục điện thế nghỉ
                neuron.potential = neuron.potential * 0.9 + -70.0 * 0.1; 
                continue;
            }

            // B. Integrate (Tích lũy)
            // Tín hiệu đầu vào ngẫu nhiên dựa trên Seed của text
            let input_current = if rng.gen_bool(0.1) { 20.0 } else { 0.0 }; 
            neuron.potential = (neuron.potential * neuron.decay) + (input_current * self.weights[i]);

            // C. Fire (Kích hoạt)
            if neuron.potential >= neuron.threshold {
                neuron.potential = -85.0; // Hyperpolarization (Quá phân cực sau khi bắn)
                neuron.refractory_timer = 5; // Nghỉ 5 chu kỳ
                active_count += 1.0;
            }
        }

        // 3. Tính toán cảm xúc dựa trên tỷ lệ kích hoạt (Firing Rate)
        let firing_rate = active_count / self.total_neurons as f32;
        let score = 1.0 + (firing_rate * 10.0);

        let mood = if score < 1.1 { "😴 Calm" }
                  else if score < 1.3 { "🙂 Happy" }
                  else if score < 1.6 { "🤔 Thinking" }
                  else { "🔥 Excited" };

        // 4. Kiểm tra bộ nhớ
        let mem_guard = self.memory.read().await;
        let reply = mem_guard.get(&text.to_lowercase())
            .cloned()
            .unwrap_or_else(|| "Tôi đang lắng nghe...".to_string());

        (score, mood.to_string(), reply)
    }

    pub async fn learn(&self, key: String, value: String) {
        let mut mem = self.memory.write().await;
        mem.insert(key.to_lowercase(), value);
    }
}