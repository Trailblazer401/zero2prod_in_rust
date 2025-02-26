//! src/routes/login/post.rs

use actix_web::{HttpResponse, web};
use actix_web::http::header::LOCATION;
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::{validate_credentials, Credentails};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
    name = "Logging in",
    skip(form, pool),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty,
    )
)]
pub async fn login(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let credential = Credentails {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credential.username));
    match validate_credentials(credential, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            HttpResponse::SeeOther().insert_header((LOCATION, "/")).finish()
        }

        Err(_) => {
            todo!()
        }
    }
}