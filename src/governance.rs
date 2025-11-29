use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub status: String, // "Active", "Passed", "Rejected"
}

pub struct NeuroDAO {
    proposals: RwLock<HashMap<u64, Proposal>>,
    next_id: RwLock<u64>,
}

impl NeuroDAO {
    pub fn new() -> Self {
        println!("⚖️  NEURO DAO GOVERNANCE SYSTEM ONLINE");
        Self {
            proposals: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }

    pub async fn create_proposal(&self, title: String, desc: String) -> u64 {
        let mut id_lock = self.next_id.write().await;
        let id = *id_lock;
        *id_lock += 1;

        let prop = Proposal {
            id,
            title,
            description: desc,
            votes_yes: 0,
            votes_no: 0,
            status: "Active".to_string(),
        };

        self.proposals.write().await.insert(id, prop);
        id
    }

    pub async fn vote(&self, id: u64, approve: bool) -> Result<String, String> {
        let mut props = self.proposals.write().await;
        if let Some(p) = props.get_mut(&id) {
            if p.status != "Active" {
                return Err("Proposal is closed".to_string());
            }
            if approve { p.votes_yes += 1; } else { p.votes_no += 1; }
            
            // Logic đơn giản: Nếu quá 10 phiếu thì chốt
            if p.votes_yes + p.votes_no >= 10 {
                p.status = if p.votes_yes > p.votes_no { "Passed".to_string() } else { "Rejected".to_string() };
            }
            Ok(format!("Voted. Current: {} Yes / {} No", p.votes_yes, p.votes_no))
        } else {
            Err("Proposal not found".to_string())
        }
    }

    pub async fn list_proposals(&self) -> Vec<Proposal> {
        let props = self.proposals.read().await;
        props.values().cloned().collect()
    }
}
