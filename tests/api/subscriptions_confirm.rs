//! tests/api/subscriptions_confirm.rs

use crate::helpers::spawn_app;
use wiremock::{
    ResponseTemplate, 
    Mock,
    matchers::{path, method},
};


#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = spawn_app().await;

    let reponse = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(reponse.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = app.get_confirmation_links(&email_request);
    
    let reponse = reqwest::get(confirmation_link.html)
        .await
        .unwrap();

    assert_eq!(reponse.status().as_u16(), 200);
}