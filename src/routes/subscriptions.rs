//! src/routes/subscriptions.rs

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]    // 该处的属性宏#[derive()]用于自动为 FormData 结构体实现来自serde库的 trait: serde::Deserialize
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber...",
    skip(form, pool),
    fields(
        // request_id = %Uuid::new_v4(),
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // let requset_id = Uuid::new_v4();
    // tracing::info!("Request id:{} - Adding '{}:{}' as a new subscriber...", requset_id, form.name, form.email);
    // let request_span = tracing::info_span!(
    //     "Adding a new subscriber...",
    //     %requset_id,
    //     subscriber_name = %form.name,
    //     subscriber_email = %form.email
    // );
    // let _request_span_guard = request_span.enter();

    // let query_span = tracing::info_span!("Saving new subscriber details into database...");
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        // name: SubscriberName::parse(form.0.name).expect("Name validation failed."),
        name,
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => {
            // tracing::info!("Request id:{} - New subscriber details have been saved successfully.", requset_id);
            HttpResponse::Ok().finish()
        }
        Err(_) => {
            // println!("Failed to execute query: {}", e);
            // tracing::error!("Request id:{} - Failed to execute query: {:?}", requset_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
    // HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details into database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,   //使用 r#"..."# 包裹SQL查询，即使用原始字符串字面量定义查询语句，这样在SQL命令中不需要进行特殊字符的转义
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)    
    // 若 subscribe 函数保留 PgConnection 作为参数，则不满足此处execute方法要求参数实现 Executor trait，
    //PgConnection类型的可变引用实现了该 trait（可变引用的唯一性保证同时只能存在一个在该Postgres连接上的查询），但 web::Data 无法提供对原类型的可变引用
    // 使用PgPool类型通过内部可变性实现共享引用
    .await
    .map_err(|e| {    // 此处闭包捕获 sqlx::query!(...).await 返回的 Err(e) 并将其所有权转移至闭包内（基于FnOnce trait实现）（若结果是Err的话）
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}