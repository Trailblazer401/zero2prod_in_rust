//! src/authentication.rs

use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use anyhow::Context;
use crate::telemetry::spawn_blocking_with_tracing;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentails {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(
    name = "Validate credentials",
    skip(credentials, pool)
)]
pub async fn validate_credentials(
    credentials: Credentails,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_passwd_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        e3NzaWdub3cfjdfkslafdjslkjflejoi$\
        HIOFSOnhofdihaohfdojLSHFL/2389%&(*(^*&%$DRYTFY%&^".to_string(),
    );

    if let Some((stored_user_id, stored_passwd_hash)) = get_stored_credentials(
            &credentials.username,
            &pool
        )
        .await?
        // .map_err(PublishError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_passwd_hash = stored_passwd_hash;
    }

    // let expected_passwd_hash = PasswordHash::new(&expected_passwd_hash.expose_secret())
    //     .context("Failed to parse password hash in PHC string format.")
    //     .map_err(PublishError::UnexpectedError)?;

    // tracing::info_span!("Verifying passwd hash").in_scope(|| {
    //     Argon2::default().verify_password(
    //         credentials.password.expose_secret().as_bytes(),
    //         &expected_passwd_hash
    //     )
    // })
    // .context("Invalid password")
    // .map_err(PublishError::AuthError)?;

    spawn_blocking_with_tracing(move || {
        verify_passwd_hash(
            expected_passwd_hash, 
            credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")??;

    // Ok(user_id)
    user_id
        .ok_or_else(|| anyhow::anyhow!("Invalid username"))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Get stored credentials",
    skip(username, pool)
)]
async fn get_stored_credentials(username: &str, pool: &PgPool) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to retrieve stored credentials")?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));
    
    Ok(row)
}

#[tracing::instrument(
    name = "Verify passwd hash",
    skip(expected_passwd_hash, passwd_candidate)
)]
fn verify_passwd_hash(
    expected_passwd_hash: Secret<String>,
    passwd_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_passwd_hash = PasswordHash::new(expected_passwd_hash.expose_secret())
        .context("Failed to parse password hash in PHC string format.")?;
        // .map_err(PublishError::UnexpectedError)?;

    Argon2::default().verify_password(
        passwd_candidate.expose_secret().as_bytes(), 
        &expected_passwd_hash,
    )
    .context("Invalid password")
    .map_err(AuthError::InvalidCredentials)
}