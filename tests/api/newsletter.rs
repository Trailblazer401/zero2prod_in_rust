//! tests/api/newsletter.rs

use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
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
            "text": "newsletter content",
            "html": "<p>newsletter content</p>"
        }
    });
    let reponse = app.post_newsletters(newsletters_request_body)
        .await;

    // Assert
    assert_eq!(reponse.status().as_u16(), 200);
}

async fn create_pending_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create Pending Subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app.email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_pending_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletters_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": {
            "text": "newsletter content",
            "html": "<p>newsletter content</p>"
        }
    });
        let reponse = app.post_newsletters(newsletters_request_body).await;

    // Assert
    assert_eq!(reponse.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (serde_json::json!({
            "content": {
                "text": "newsletter content",
                "html": "<p>newsletter content</p>"
            }
        }), "missing title"),
        (serde_json::json!({"title": "a"}), "missing content"),
        (serde_json::json!({"content": {}}), "missing title"),
        (serde_json::json!({"title": "a", "content": {}}), "missing content"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let reponse = app.post_newsletters(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            reponse.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn reject_requests_missing_authorization() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let reponse = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "newsletter title",
            "content": {
                "text": "newsletter content",
                "html": "<p>newsletter content</p>"
            }
        }))
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(401, reponse.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        reponse.headers()["www-authenticate"]
    )
}