//! tests/api/login.rs

use crate::helpers::{spawn_app, assert_is_redirect_to};
// use reqwest::header::HeaderValue;
// use std::collections::HashSet;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-passwd"
    });
    let reponse = app.post_login(&login_body).await;

    assert_eq!(reponse.status().as_u16(), 303);
    assert_is_redirect_to(&reponse, "/login");

    // let flash_cookie = reponse.cookies().find(|c| c.name() == "_flash").unwrap();
    // assert_eq!(flash_cookie.value(), "Authentication failed");

    let html = app.get_login_html().await;
    assert!(html.contains(r#"<p><i>Authentication failed</i></p>"#));

    let html = app.get_login_html().await;
    assert!(!html.contains(r#"<p><i>Authentication failed</i></p>"#));
}