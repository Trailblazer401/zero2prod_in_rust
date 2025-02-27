//! src/routes/newsletters.rs
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use actix_web::{web, HttpRequest};
use actix_web::http::header::{HeaderMap, HeaderValue, WWW_AUTHENTICATE};
use secrecy::Secret;
use sqlx::PgPool;
use crate::domain::SubscriberEmail;
use crate::routes::error_chain_fmt;
use crate::email_client::EmailClient;
use anyhow::Context;
use base64::Engine;

use crate::authentication::AuthError;
use crate::authentication::{Credentails, validate_credentials};

#[derive(serde::Deserialize)]
pub struct NewsletterBody {
    title: String,
    content: NewsletterContent,
}

#[derive(serde::Deserialize)]
pub struct NewsletterContent {
    text: String,
    html: String,
}

struct ConfirmedSubscriber {
    // id: String,
    // email: String,
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authorization failed")]
    AuthError(#[source] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    // fn status_code(&self) -> StatusCode {
    //     match self {
    //         Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         Self::AuthError(_) => StatusCode::UNAUTHORIZED,
    //     }
    // }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response.headers_mut().insert(WWW_AUTHENTICATE, header_value);
                response
            }
        }
        .into()
    }
}

// struct Credentails {
//     username: String,
//     password: Secret<String>,
// }

fn basic_authentication(headers: &HeaderMap) -> Result<Credentails, anyhow::Error> {
    let authorization_header = headers
        .get("Authorization")
        .context("Missing 'Authorization' header")?
        .to_str()
        .context("Failed to parse authorization header: not a valid UTF-8 string")?;
    let base64encoded = authorization_header
        .strip_prefix("Basic ")
        .context("Invalid authorization header: not 'Basic' scheme")?;
    // let decoded_bytes = base64::decode_config(base64encoded, base64::STANDARD)
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64encoded)
        .context("Failed to decode base64-encoded 'Basic' credentials")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("Decoded credentials are not valid UTF-8")?;

    let mut credential_split = decoded_credentials.splitn(2, ':');
    let username = credential_split
        .next()
        .ok_or_else(|| anyhow::anyhow!("Username is missing in the 'Basic' authorization credentials"))?
        .to_string();
    let password = credential_split
        .next()
        .ok_or_else(|| anyhow::anyhow!("Password is missing in the 'Basic' authorization credentials"))?
        .to_string();

    Ok(Credentails {
        username,
        password: Secret::new(password),
    })
}

#[tracing::instrument(
    name = "Publishing a newsletter",
    skip(newsletter_body, pool, email_client, request),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    newsletter_body: web::Json<NewsletterBody>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers())
        .map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    let subscribers = get_confirmed_subscribers(&pool)
        .await?;
        // .expect("Failed to retrieve confirmed subscribers");
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client.send_email(
                    &subscriber.email,
                    &newsletter_body.title,
                    &newsletter_body.content.html,
                    &newsletter_body.content.text,
                )
                .await
                .with_context(|| format!("Failed to send newsletter to {}", subscriber.email))?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber because of invalid email address"
                );
            }
            
        }
    }
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Get confirmed subscribers",
    skip(pool)
)]
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| match SubscriberEmail::parse(row.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error))
    })
    .collect();    
    
    Ok(confirmed_subscribers)
}

// #[tracing::instrument(
//     name = "Validate credentials",
//     skip(credentials, pool)
// )]
// async fn validate_credentials(
//     credentials: Credentails,
//     pool: &PgPool,
// ) -> Result<uuid::Uuid, AuthError> {
//     let mut user_id = None;
//     let mut expected_passwd_hash = Secret::new(
//         "$argon2id$v=19$m=15000,t=2,p=1$\
//         e3NzaWdub3cfjdfkslafdjslkjflejoi$\
//         HIOFSOnhofdihaohfdojLSHFL/2389%&(*(^*&%$DRYTFY%&^".to_string(),
//     );

//     if let Some((stored_user_id, stored_passwd_hash)) = get_stored_credentials(
//             &credentials.username,
//             &pool
//         )
//         .await?
//     {
//         user_id = Some(stored_user_id);
//         expected_passwd_hash = stored_passwd_hash;
//     }

//     spawn_blocking_with_tracing(move || {
//         verify_passwd_hash(
//             expected_passwd_hash, 
//             credentials.password)
//     })
//     .await
//     .context("Failed to spawn blocking task")??;

//     user_id
//         .ok_or_else(|| anyhow::anyhow!("Invalid username"))
//         .map_err(AuthError::InvalidCredentials)
// }

// #[tracing::instrument(
//     name = "Get stored credentials",
//     skip(username, pool)
// )]
// async fn get_stored_credentials(username: &str, pool: &PgPool) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
//     let row = sqlx::query!(
//         r#"
//         SELECT user_id, password_hash
//         FROM users
//         WHERE username = $1
//         "#,
//         username,
//     )
//     .fetch_optional(pool)
//     .await
//     .context("Failed to retrieve stored credentials")?
//     .map(|row| (row.user_id, Secret::new(row.password_hash)));
    
//     Ok(row)
// }

// #[tracing::instrument(
//     name = "Verify passwd hash",
//     skip(expected_passwd_hash, passwd_candidate)
// )]
// fn verify_passwd_hash(
//     expected_passwd_hash: Secret<String>,
//     passwd_candidate: Secret<String>,
// ) -> Result<(), AuthError> {
//     let expected_passwd_hash = PasswordHash::new(expected_passwd_hash.expose_secret())
//         .context("Failed to parse password hash in PHC string format.")?;

//     Argon2::default().verify_password(
//         passwd_candidate.expose_secret().as_bytes(), 
//         &expected_passwd_hash,
//     )
//     .context("Invalid password")
//     .map_err(AuthError::InvalidCredentials)
// }