use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RiskQueryRequest {
    pub vegetation_density: f64,
    pub invasive_grass_percent: f64,
    pub slope_degrees: f64,
}

#[derive(Serialize)]
pub struct RiskResponse {
    pub risk_index: f64,
    pub defensible_zone_m: (u32, u32, u32),
    pub recommendation: String,
}

async fn compute_risk(req: web::Json<RiskQueryRequest>) -> HttpResponse {
    // Use cybersomething::core::math::risk_index::RiskCalculator
    HttpResponse::Ok().json(RiskResponse {
        risk_index: 0.65,
        defensible_zone_m: (100, 200, 20),
        recommendation: "Medium risk: implement 10-30m defensible space".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .route("/health", web::get().to(|| async { HttpResponse::Ok().finish() }))
            .route("/api/v1/risk", web::post().to(compute_risk))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
