use actix_web::{App, HttpServer, web, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::NamedFile;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;
use libp2p::identity;
use tokio::time::{sleep, Duration};

mod snn_core;
mod block;
mod chain;
mod ethics;
mod storage;
mod p2p;
mod quantum;
mod trainer;
mod deploy;
mod oracle;
mod llm;
mod cache;
mod governance;
mod webnode;
mod transaction;
mod wallet;

use chain::PappapChain;
use ethics::EthicsFilter;
use p2p::P2PNode;
use trainer::AutoTrainer;
use deploy::trigger_deploy;
use governance::NeuroDAO;
use storage::Storage;
use cache::SmartCache;
use webnode::WebNodeManager;
use std::sync::atomic::{AtomicUsize, Ordering};
use transaction::Transaction;

#[derive(Deserialize)]
struct PromptReq { prompt: String }
#[derive(Deserialize)]
struct TeachReq { keyword: String, answer: String }
#[derive(Deserialize)]
struct ComputeReq { client_id: String, data_payload: String }
#[derive(Deserialize)]
struct ProposalReq { title: String, desc: String }
#[derive(Deserialize)]
struct VoteReq { id: u64, approve: bool }
#[derive(Deserialize)]
struct PingReq { client_id: String, hashrate: f32 }
#[derive(Deserialize)]
struct TxReq { sender: String, receiver: String, amount: u64, signature: String, id: String } 

async fn get_blocks(data: web::Data<Arc<PappapChain>>) -> impl Responder {
    let recent = data.storage.get_recent_blocks(15);
    HttpResponse::Ok().json(recent)
}
async fn analyze_prompt(data: web::Data<Arc<PappapChain>>, req: web::Json<PromptReq>) -> impl Responder {
    if let Err(e) = EthicsFilter::check(&req.prompt) { return HttpResponse::BadRequest().json(json!({"error": e})); }
    let (score, mood, reply) = data.snn.process_text(&req.prompt).await;
    HttpResponse::Ok().json(json!({ "prompt": req.prompt, "mood": mood, "ai_response": reply, "spike": score }))
}
async fn teach_ai(data: web::Data<Arc<PappapChain>>, req: web::Json<TeachReq>) -> impl Responder {
    if let Err(e) = EthicsFilter::check(&req.keyword) { return HttpResponse::BadRequest().json(json!({"error": e})); }
    data.snn.learn(req.keyword.clone(), req.answer.clone()).await;
    HttpResponse::Ok().json(json!({ "status": "Saved", "key": req.keyword }))
}
async fn compute_task(data: web::Data<Arc<PappapChain>>, req: web::Json<ComputeReq>) -> impl Responder {
    if let Err(e) = EthicsFilter::check(&req.data_payload) { return HttpResponse::BadRequest().json(json!({"error": e})); }
    let (score, mood, _) = data.snn.process_text(&req.data_payload).await;
    HttpResponse::Ok().json(json!({ "provider": "PAPPAP", "result": { "spike": score, "mood": mood }, "status": "SUCCESS" }))
}
async fn health(data: web::Data<Arc<PappapChain>>, wn: web::Data<Arc<WebNodeManager>>, pc: web::Data<Arc<AtomicUsize>>) -> impl Responder {
    let h = *data.height.read().await;
    let (n, p) = data.snn.stats().await;
    let stats = data.storage.load_stats();
    let (wc, wp) = wn.get_stats().await;
    HttpResponse::Ok().json(json!({ 
        "height": h, "power": p + wp, "neurons": n + (wc * 100), 
        "total_mined": stats.total_blocks, "web_nodes": wc, "p2p_nodes": pc.load(Ordering::Relaxed) 
    }))
}
async fn index() -> impl Responder { NamedFile::open_async("./static/index.html").await }
async fn create_proposal(dao: web::Data<Arc<NeuroDAO>>, req: web::Json<ProposalReq>) -> impl Responder {
    let id = dao.create_proposal(req.title.clone(), req.desc.clone()).await;
    HttpResponse::Ok().json(json!({ "status": "Created", "id": id }))
}
async fn vote_proposal(dao: web::Data<Arc<NeuroDAO>>, req: web::Json<VoteReq>) -> impl Responder {
    match dao.vote(req.id, req.approve).await { Ok(m) => HttpResponse::Ok().json(m), Err(e) => HttpResponse::BadRequest().json(e) }
}
async fn list_proposals(dao: web::Data<Arc<NeuroDAO>>) -> impl Responder { HttpResponse::Ok().json(dao.list_proposals().await) }
async fn webnode_ping(wn: web::Data<Arc<WebNodeManager>>, req: web::Json<PingReq>) -> impl Responder {
    wn.register_beat(req.client_id.clone(), req.hashrate).await;
    HttpResponse::Ok().json(json!({ "status": "Ack" }))
}
async fn submit_tx(data: web::Data<Arc<PappapChain>>, req: web::Json<TxReq>) -> impl Responder {
    let tx = Transaction {
        id: req.id.clone(), sender: req.sender.clone(), receiver: req.receiver.clone(),
        amount: req.amount, fee: 100, nonce: 0, timestamp: 0, signature: req.signature.clone()
    };
    if data.mempool.add_tx(tx) { HttpResponse::Ok().json(json!({"status":"Success"})) } else { HttpResponse::BadRequest().json(json!({"status":"Fail"})) }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let storage = Arc::new(Storage::new("pappap_data"));
    let mut stats = storage.load_stats();
    stats.total_starts += 1;
    storage.save_stats(&stats);

    let local_key = if let Some(kb) = storage.load_node_secret() { identity::Keypair::from_protobuf_encoding(&kb).expect("Key Error") } else { let k = identity::Keypair::generate_ed25519(); storage.save_node_secret(&k.to_protobuf_encoding().unwrap()); k };

    let cache = SmartCache::new();
    let dao = Arc::new(NeuroDAO::new());
    let wn_mgr = Arc::new(WebNodeManager::new());
    let p_count = Arc::new(AtomicUsize::new(0));

    let (p2p_node, _, pid) = P2PNode::new(local_key, p_count.clone()).await.unwrap();
    let p2p_arc = Arc::new(Mutex::new(p2p_node));
    let p2p_run = p2p_arc.clone();
    let p2p_chain = p2p_arc.clone(); // Clone d? truy?n vào chain

    // S?A L?I: Truy?n d? 3 tham s? (storage, cache, p2p_chain)
    let chain = Arc::new(PappapChain::new(storage.clone(), cache, p2p_chain).await);
    
    let m_chain = chain.clone();
    let t_snn = chain.snn.clone();
    let wn_run = wn_mgr.clone();

    println!("?? NODE STARTED | PID: {}", pid);

    tokio::spawn(async move { let mut n = p2p_run.lock().await; n.run().await; });
    tokio::spawn(async move { m_chain.run().await; });
    tokio::spawn(async move { AutoTrainer::start(t_snn).await; });
    tokio::spawn(async move { loop { sleep(Duration::from_secs(10)).await; wn_run.prune_offline().await; } });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chain.clone()))
            .app_data(web::Data::new(dao.clone()))
            .app_data(web::Data::new(wn_mgr.clone()))
            .app_data(web::Data::new(p_count.clone()))
            .wrap(Cors::permissive())
            .route("/", web::get().to(index))
            .route("/api/health", web::get().to(health))
            .route("/api/blocks", web::get().to(get_blocks))
            .route("/api/prompt", web::post().to(analyze_prompt))
            .route("/api/teach", web::post().to(teach_ai))
            .route("/api/compute", web::post().to(compute_task))
            .route("/api/deploy", web::post().to(trigger_deploy))
            .route("/api/dao/create", web::post().to(create_proposal))
            .route("/api/dao/vote", web::post().to(vote_proposal))
            .route("/api/dao/list", web::get().to(list_proposals))
            .route("/api/webnode/ping", web::post().to(webnode_ping))
            .route("/api/tx/submit", web::post().to(submit_tx))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
