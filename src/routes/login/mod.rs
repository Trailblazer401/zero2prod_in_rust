//! src/routes/login/mod.rs

// use actix_web::HttpResponse;
// use actix_web::http::header::ContentType;

// pub async fn login_form() -> HttpResponse {
//     HttpResponse::Ok()
//         .content_type(ContentType::html())
//         .body(include_str!("login.html"))
// }

mod post;
mod get;

pub use post::login;
pub use get::login_form;