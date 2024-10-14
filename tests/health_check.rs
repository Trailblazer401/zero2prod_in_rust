//! tests/health_check.rs

#[tokio::test]
async fn health_check_works() {
    spawn_app();
    let client = reqwest::Client::new();

    let reponse = client
        .get("http://127.0.0.1:8888/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(reponse.status().is_success());
    assert_eq!(Some(0), reponse.content_length());
}

fn spawn_app() {
    // zero2prod::run().await
    let server = zero2prod::run().expect("Failed to bind addr");
    let _ = tokio::spawn(server);
}