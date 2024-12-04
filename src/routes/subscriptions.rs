//! src/routes/subscriptions.rs

use actix_web::{web, HttpResponse};
use actix_web::http::StatusCode;
use sqlx::{Executor, PgPool};
use chrono::Utc;
use uuid::Uuid;
use crate::{domain::{NewSubscriber, SubscriberEmail, SubscriberName}, email_client::EmailClient, startup::ApplicationBaseUrl};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Postgres, Transaction};
use actix_web::ResponseError;

#[derive(serde::Deserialize)]    // 该处的属性宏#[derive()]用于自动为 FormData 结构体实现来自serde库的 trait: serde::Deserialize
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber{name, email})
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber...",
    skip(form, pool, email_client, base_url),
    fields(
        // request_id = %Uuid::new_v4(),
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into()?;  // web::Form<T> 实际上是一个包含一泛型的元祖结构体，即 struct Form<T>(T)， 使用.0访问其第一个字段 T
    
    let mut transaction= pool.begin().await?;
    
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber).await?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;
    
    transaction.commit().await?;

    send_confirmation_email(
        &email_client, 
        new_subscriber, 
        &base_url.0,
        &subscription_token,
    ).await?;
    
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details into database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,   //使用 r#"..."# 包裹SQL查询，即使用原始字符串字面量定义查询语句，这样在SQL命令中不需要进行特殊字符的转义
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );
    transaction
        .execute(query)    
        // 若 subscribe 函数保留 PgConnection 作为参数，则不满足此处execute方法要求参数实现 Executor trait，
        // PgConnection类型的可变引用实现了该 trait（可变引用的唯一性保证同时只能存在一个在该Postgres连接上的查询），但 web::Data 无法提供对原类型的可变引用
        // 使用PgPool类型通过内部可变性实现共享引用
        .await
        .map_err(|e| {    // 此处闭包捕获 sqlx::query!(...).await 返回的 Err(e) 并将其所有权转移至闭包内（基于FnOnce trait实现）（若结果是Err的话）
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Sending new subscriber a confirmation email",
    skip(email_client, 
        new_subscriber, 
        base_url, 
        subscription_token
    )
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}", 
        base_url, 
        subscription_token
    );

    let plain_body = format!(
        "Welcome to our newsletter!\n
        Visit {} to confirm your subscription.", 
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.", 
        confirmation_link
    );

    email_client.send_email(
        new_subscriber.email,
        "Welcome",
        &html_body,
        &plain_body,
    ).await
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token into database...",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens 
        (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );
    transaction
        .execute(query)
        .await
        .map_err(|e| {
            // tracing::error!("Failed to execute query: {:?}", e);
            StoreTokenError(e)
        })?;
        Ok(())
}

// #[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);

// impl ResponseError for StoreTokenError {}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}\nCaused by:\n\t{}", self, self.0)
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f,"Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(Debug)]
pub enum SubscribeError {
    ValidationError(String),
    DatabaseError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to create a new subscriber."
        )
    }
}

impl std::error::Error for SubscribeError {}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::DatabaseError(_) |
            SubscribeError::SendEmailError(_) |
            SubscribeError::StoreTokenError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(value: reqwest::Error) -> Self {
        Self::SendEmailError(value)
    }
}

impl From<sqlx::Error> for SubscribeError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(value: StoreTokenError) -> Self {
        Self::StoreTokenError(value)
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        Self::ValidationError(value)
    }
}