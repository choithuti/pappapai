use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
pub struct DeployReq {
    secret_key: String,
}

pub async fn trigger_deploy(req: web::Json<DeployReq>) -> impl Responder {
    // MẬT KHẨU DEPLOY (Bạn có thể đổi nếu muốn)
    let my_secret = "pappap_super_secret_deploy_key_2025"; 

    if req.secret_key != my_secret {
        return HttpResponse::Unauthorized().json("❌ Sai mật khẩu deploy!");
    }

    println!("⚠️  DEPLOY TRIGGERED BY ADMIN API...");

    // Gọi script deploy.sh
    let _ = Command::new("nohup")
        .arg("./deploy.sh")
        .spawn();

    HttpResponse::Ok().json("✅ Deployment Started! Server will restart shortly.")
}
