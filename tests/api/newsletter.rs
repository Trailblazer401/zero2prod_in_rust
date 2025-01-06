//! tests/api/newsletter.rs

use crate::helpers::{spawn_app, TestApp};
use wiremock::matchers::{method, path, any};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_pending_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_pending_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletters_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": {
            "test": "newsletter content",
            "html": "<p>newsletter content</p>"
        }
    });
    let reponse = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletters_request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(reponse.status().as_u16(), 200);
}

async fn create_pending_subscriber(app: &TestApp) {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(201))
        .named("Create Pending Subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}