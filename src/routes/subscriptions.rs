//! src/routes/subscriptions.rs

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::{domain::{NewSubscriber, SubscriberEmail, SubscriberName}, email_client::EmailClient, startup::ApplicationBaseUrl};

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
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {  // web::Form<T> 实际上是一个包含一泛型的元祖结构体，即 struct Form<T>(T)， 使用.0访问其第一个字段 T
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(&email_client, new_subscriber, &base_url.0).await.is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details into database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,   //使用 r#"..."# 包裹SQL查询，即使用原始字符串字面量定义查询语句，这样在SQL命令中不需要进行特殊字符的转义
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)    
    // 若 subscribe 函数保留 PgConnection 作为参数，则不满足此处execute方法要求参数实现 Executor trait，
    // PgConnection类型的可变引用实现了该 trait（可变引用的唯一性保证同时只能存在一个在该Postgres连接上的查询），但 web::Data 无法提供对原类型的可变引用
    // 使用PgPool类型通过内部可变性实现共享引用
    .await
    .map_err(|e| {    // 此处闭包捕获 sqlx::query!(...).await 返回的 Err(e) 并将其所有权转移至闭包内（基于FnOnce trait实现）（若结果是Err的话）
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Sending new subscriber a confirmation email",
    skip(email_client, new_subscriber, base_url)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscriptions_token=atoken", base_url);

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