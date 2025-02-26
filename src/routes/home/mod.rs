//! src/routes/home/mod.rs

use actix_web::HttpResponse;

pub async fn home() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("home.html"))
}