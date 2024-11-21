//! tests/api/subscriptions.rs

use wiremock::{matchers::{method, path}, Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_valid() {
    // let app_addr = spawn_app();
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let reponse = test_app.post_subscriptions(body.into()).await;

    assert_eq!(200,reponse.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_bad() {
    // let app_addr = spawn_app();
    let test_app = spawn_app().await;
    let test_case = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email")
    ];
    
    for (invalid_body, error_msg) in test_case {
        let reponse = test_app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            reponse.status().as_u16(),
            "The API did not fail with 400 on requst payload: \"{}\"",
            error_msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let test_case = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=not_an_email", "invalid email"),
    ];

    for (body, description) in test_case {
        let reponse = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            reponse.status().as_u16(),
            "The API didnt return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}