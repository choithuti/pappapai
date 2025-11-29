use actix_web::{App, HttpServer, web, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::NamedFile;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;
use libp2p::identity;

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

use chain::PappapChain;
use ethics::EthicsFilter;
use p2p::P2PNode;
use trainer::AutoTrainer;
use deploy::trigger_deploy;
use governance::NeuroDAO;
use storage::Storage;

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

// --- API Handlers (Giữ nguyên logic) ---
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
async fn health(data: web::Data<Arc<PappapChain>>) -> impl Responder {
    let h = *data.height.read().await;
    let (n, p) = data.snn.stats().await;
    let stats = data.storage.load_stats();
    HttpResponse::Ok().json(json!({ 
        "height": h, "power": p, "neurons": n, 
        "total_mined": stats.total_blocks,
        "reputation": stats.reputation,
        "starts": stats.total_starts
    }))
}
async fn index() -> impl Responder { NamedFile::open_async("./static/index.html").await }
async fn create_proposal(dao: web::Data<Arc<NeuroDAO>>, req: web::Json<ProposalReq>) -> impl Responder {
    let id = dao.create_proposal(req.title.clone(), req.desc.clone()).await;
    HttpResponse::Ok().json(json!({ "status": "Created", "id": id }))
}
async fn vote_proposal(dao: web::Data<Arc<NeuroDAO>>, req: web::Json<VoteReq>) -> impl Responder {
    match dao.vote(req.id, req.approve).await {
        Ok(msg) => HttpResponse::Ok().json(json!({ "status": "Voted", "msg": msg })),
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": e }))
    }
}
async fn list_proposals(dao: web::Data<Arc<NeuroDAO>>) -> impl Responder {
    HttpResponse::Ok().json(dao.list_proposals().await)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. KẾT NỐI STORAGE & QUẢN LÝ DANH TÍNH (IDENTITY)
    let storage = Arc::new(Storage::new("pappap_data"));
    
    // Cập nhật thống kê khởi động
    let mut stats = storage.load_stats();
    stats.total_starts += 1;
    storage.save_stats(&stats);

    // Load hoặc Tạo mới Identity
    let local_key = if let Some(key_bytes) = storage.load_node_secret() {
        identity::Keypair::from_protobuf_encoding(&key_bytes).expect("Invalid Key")
    } else {
        println!("🔑 GENERATING NEW IDENTITY...");
        let new_key = identity::Keypair::generate_ed25519();
        let key_bytes = new_key.to_protobuf_encoding().unwrap();
        storage.save_node_secret(&key_bytes);
        new_key
    };

    // 2. KHỞI TẠO P2P VỚI IDENTITY ĐÃ LƯU
    let (p2p_node, _, local_peer_id) = P2PNode::new(local_key).await.expect("Failed P2P");
    let p2p_node = Arc::new(Mutex::new(p2p_node));
    let p2p_run = p2p_node.clone();

    // 3. HIỂN THỊ THÔNG TIN NODE (BANNER)
    println!("\n==================================================");
    println!("💎 PAPPAP AI CHAIN | GENESIS NODE v0.5");
    println!("==================================================");
    println!("🆔 PEER ID    : {}", local_peer_id);
    println!("💾 STORAGE    : Sled (Persistent)");
    println!("📊 STARTS     : {} times", stats.total_starts);
    println!("🧱 MINED BLOCKS: {}", stats.total_blocks);
    println!("🌟 REPUTATION : {}", stats.reputation);
    println!("==================================================\n");

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 { p2p_node.lock().await.dial(&args[1]); }

    // Dùng lại storage cũ cho Chain để đồng bộ
    // Sửa chain::new để nhận storage đã init
    // (Ở đây ta tạm thời tạo mới trong chain nếu chưa sửa chain.rs, 
    // nhưng tốt nhất là sửa chain.rs để nhận Storage Arc.
    // Để code chạy ngay, ta dùng Chain::new() hiện tại và nó sẽ tự mở lại db path cũ)
    let chain = Arc::new(PappapChain::new().await); 
    
    let mining_chain = chain.clone();
    let trainer_snn = chain.snn.clone();
    let dao = Arc::new(NeuroDAO::new());

    tokio::spawn(async move { let mut n = p2p_run.lock().await; n.run().await; });
    tokio::spawn(async move { mining_chain.run().await; });
    tokio::spawn(async move { AutoTrainer::start(trainer_snn).await; });

    println!("🚀 HTTP SERVER : http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chain.clone()))
            .app_data(web::Data::new(dao.clone()))
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
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
