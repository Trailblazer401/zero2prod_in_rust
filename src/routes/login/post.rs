//! src/routes/login/post.rs

use actix_web::error::InternalError;
use actix_web::{web, HttpResponse};
use actix_web::http::header::LOCATION;
use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use hmac::{Hmac, Mac};

use crate::authentication::{validate_credentials, AuthError, Credentails};
use crate::routes::error_chain_fmt;
use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// impl ResponseError for LoginError {
//     fn status_code(&self) -> StatusCode {
//         // match self {
//         //     Self::AuthError(_) => StatusCode::UNAUTHORIZED,
//         //     Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
//         // }
//         StatusCode::SEE_OTHER
//     }

//     fn error_response(&self) -> HttpResponse {
//         let query_string = format!(
//             "error={}",
//             urlencoding::Encoded::new(self.to_string())
//         );
//         let secret: &[u8] = todo!("get key");
//         let hmac_tag = {
//             let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
//             mac.update(query_string.as_bytes());
//             mac.finalize().into_bytes()
//         };
//         HttpResponse::build(self.status_code())
//             .insert_header((LOCATION, format!("/login?error={query_string}&tag={hmac_tag:x}")))
//             .finish()
//     }
// }

#[tracing::instrument(
    name = "Logging in",
    skip(form, pool, secret),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty,
    )
)]
pub async fn login(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credential = Credentails {
        username: form.0.username,
        password: form.0.password,
    };
    // let user_id = validate_credentials(credential, &pool)
    //     .await
    //     .map_err(|e| match e{
    //         AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
    //         AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
    //     })?;

    // tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    
    match validate_credentials(credential, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Ok(HttpResponse::SeeOther().insert_header((LOCATION, "/")).finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                    secret.0.expose_secret().as_bytes()
                ).unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };
            let response = HttpResponse::SeeOther()
                .insert_header((
                    LOCATION, 
                    format!("/login?{}&tag={:x}", query_string, hmac_tag),
                ))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
    // Ok(HttpResponse::SeeOther().insert_header((LOCATION, "/")).finish())
}