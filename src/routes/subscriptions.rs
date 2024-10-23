//! src/routes/subscriptions.rs

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing::Instrument;

#[derive(serde::Deserialize)]    // 该处的属性宏#[derive()]用于自动为 FormData 结构体实现来自serde库的 trait: serde::Deserialize
pub struct FormData {
    email: String,
    name: String
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let requset_id = Uuid::new_v4();
    // tracing::info!("Request id:{} - Adding '{}:{}' as a new subscriber...", requset_id, form.name, form.email);
    let request_span = tracing::info_span!(
        "Adding a new subscriber...",
        %requset_id,
        subscriber_name = %form.name,
        subscriber_email = %form.email
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details into database...");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,   //使用 r#"..."# 包裹SQL查询，即使用原始字符串字面量定义查询语句，这样在SQL命令中不需要进行特殊字符的转义
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())    // 若 subscribe 函数保留 PgConnection 作为参数，则不满足此处execute方法要求参数实现 Executor trait，PgConnection类型的可变引用实现了该 trait（可变引用的唯一性保证同时只能存在一个在该Postgres连接上的查询），但 web::Data 无法提供对原类型的可变引用
    .instrument(query_span)
    .await    // 使用PgPool类型通过内部可变性实现共享引用
    {
        Ok(_) => {
            // tracing::info!("Request id:{} - New subscriber details have been saved successfully.", requset_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // println!("Failed to execute query: {}", e);
            tracing::error!("Request id:{} - Failed to execute query: {:?}", requset_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
    // HttpResponse::Ok().finish()
}