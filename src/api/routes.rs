// src/api/routes.rs
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use serde_json::json;

// Import đúng đường dẫn mới
use crate::core::snn::SNNCore; 
use crate::services::knowledge;
// Bỏ 'voice' nếu chưa dùng để tránh warning, hoặc để lại nếu sắp dùng

pub fn config_services(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/prompt").route(web::post().to(prompt_handler)))
       .service(web::resource("/api/health").route(web::get().to(health_handler)));
}

async fn health_handler(snn: web::Data<Arc<SNNCore>>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "Alive",
        "neurons": snn.neuron_count().await,
        "compliance": "Vietnam Law & Global Standards"
    }))
}

async fn prompt_handler(
    snn: web::Data<Arc<SNNCore>>,
    req: web::Json<serde_json::Value>,
) -> impl Responder {
    let prompt = req["prompt"].as_str().unwrap_or("").trim();

    if let Err(e) = snn.check_compliance(prompt).await {
        return HttpResponse::BadRequest().json(json!({"error": e, "status": "Blocked"}));
    }

    let response_text = if prompt.len() > 50 || prompt.contains("tìm hiểu") {
        knowledge::auto_learn_trusted(prompt).await
    } else {
        snn.process_prompt(prompt).await
    };

    HttpResponse::Ok().json(json!({
        "response": response_text,
        "audio_base64": null
    }))
}