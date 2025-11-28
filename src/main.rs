use actix_web::{App, HttpServer, web, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::NamedFile;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

mod snn_core;
mod block;
mod chain;
mod ethics;
mod storage;
mod p2p;
mod quantum; // <-- THÊM MODULE MỚI

use chain::PappapChain;
use ethics::EthicsFilter;
use p2p::P2PNode;

// ... (Giữ nguyên toàn bộ phần code còn lại của main.rs như phiên bản trước)
#[derive(Deserialize)]
struct PromptReq { prompt: String }
#[derive(Deserialize)]
struct TeachReq { keyword: String, answer: String }
#[derive(Deserialize)]
struct ComputeReq { client_id: String, data_payload: String }

async fn get_blocks(data: web::Data<Arc<PappapChain>>) -> impl Responder {
    // API Lấy blocks từ RAM history cho nhanh
    let history = data.blocks_history.read().await;
    let recent: Vec<_> = history.iter().rev().collect();
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
    HttpResponse::Ok().json(json!({
        "provider": "PAPPAP_GENESIS", "result": { "spike": score, "mood": mood }, "status": "SUCCESS"
    }))
}

async fn health(data: web::Data<Arc<PappapChain>>) -> impl Responder {
    let h = *data.height.read().await;
    let (n, p) = data.snn.stats().await;
    HttpResponse::Ok().json(json!({ 
        "height": h, "power": p, "neurons": n, 
        "security": "Post-Quantum (Dilithium5)" 
    }))
}

async fn index() -> impl Responder { NamedFile::open_async("./static/index.html").await }

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (p2p_node, _rx, local_peer_id) = P2PNode::new().await.expect("Failed to start P2P");
    let p2p_node = Arc::new(Mutex::new(p2p_node));
    let p2p_for_run = p2p_node.clone();

    println!("🌟 GENESIS PEER ID: {}", local_peer_id);

    let chain = Arc::new(PappapChain::new().await);
    let mining_chain = chain.clone();

    tokio::spawn(async move {
        let mut node = p2p_for_run.lock().await;
        node.run().await;
    });

    tokio::spawn(async move {
        mining_chain.run().await;
    });

    println!("🚀 PUBLIC GENESIS NODE: http://0.0.0.0:8080");
    println!("🌍 P2P ADDRESS: /ip4/72.61.126.190/tcp/9000/p2p/{}", local_peer_id);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chain.clone()))
            .wrap(Cors::permissive())
            .route("/", web::get().to(index))
            .route("/api/health", web::get().to(health))
            .route("/api/blocks", web::get().to(get_blocks))
            .route("/api/prompt", web::post().to(analyze_prompt))
            .route("/api/teach", web::post().to(teach_ai))
            .route("/api/compute", web::post().to(compute_task))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
