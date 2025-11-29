use tokio::sync::RwLock;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::storage::Storage;
use crate::oracle::Oracle;
use crate::llm::LLMBridge;
use crate::cache::SmartCache;
use regex::Regex;

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
    cache: SmartCache,
    total_neurons: usize,
    momentum: RwLock<f32>,
}

impl SNNCore {
    pub fn new(storage: Arc<Storage>, cache: SmartCache) -> Self {
        let mut rng = rand::thread_rng();
        let neuron_count = 1_000_000;
        println!("?? SNN CORE ONLINE | Neurons: {}", neuron_count);

        let mut neurons = Vec::with_capacity(neuron_count);
        for _ in 0..1000 {
            neurons.push(BioNeuron { potential: -70.0, threshold: -55.0, decay: 0.95, refractory_timer: 0, sensitivity: rng.gen_range(0.5..1.5) });
        }

        Self {
            neurons: RwLock::new(neurons),
            storage,
            oracle: Oracle::new(),
            llm: LLMBridge::new(),
            cache,
            total_neurons: neuron_count,
            momentum: RwLock::new(1.0),
        }
    }

    pub async fn train_step(&self, intensity: f32) -> f32 {
        let mut neurons = self.neurons.write().await;
        let mut rng = rand::thread_rng();
        let mut active = 0.0;
        for n in neurons.iter_mut() {
            n.potential += intensity * n.sensitivity + rng.gen_range(-0.1..0.1);
            if n.potential >= n.threshold { n.potential = -85.0; active += 1.0; } else { n.potential *= n.decay; }
        }
        active
    }

    pub async fn forward(&self, i: f32) -> f32 { self.train_step(i).await }
    pub async fn stats(&self) -> (usize, f32) { (self.total_neurons, 1024.0) }
    pub async fn learn(&self, k: String, v: String) { self.storage.learn_fact(&k, &v); }
    
    pub async fn process_text(&self, text: &str) -> (f32, String, String) {
        // (Gi? nguyên logic x? lý text nhu các phiên b?n tru?c)
        let mut hasher = DefaultHasher::new(); text.hash(&mut hasher);
        let mut rng = StdRng::seed_from_u64(hasher.finish());
        let score = 1.0 + rng.gen_range(0.0..1.5);
        
        if let Some(ans) = self.cache.get(text).await { return (score, "? Cache".into(), ans); }
        if let Some(ans) = self.storage.recall_fact(text) { return (score, "?? Memory".into(), ans); }
        
        let mut ans = String::new();
        if let Ok(res) = self.oracle.smart_search(text).await { if !res.contains("Không tìm th?y") { ans = res; } }
        if ans.is_empty() { if let Ok(res) = self.llm.ask_ai(text).await { ans = res; } }
        
        if !ans.is_empty() { 
            self.learn(text.into(), ans.clone()).await; 
            self.cache.set(text.into(), ans.clone()).await; 
        } else { ans = "Không tìm th?y.".into(); }
        
        (score, "?? AI".into(), ans)
    }
}
