//! tests/health_check.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // spawn_app();
    let client = reqwest::Client::new();
    let test_app = spawn_app().await;

    let reponse = client
        .get(format!("{}/health_check", &(test_app.address)))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(reponse.status().is_success());
    assert_eq!(Some(0), reponse.content_length());
}