//! tests/health_check.rs

#[tokio::test]
async fn health_check_works() {
    spawn_app().await.expect("Failed to spawn out app.");
    let client = reqwest::Client::new();

    let reponse = client
        .get("http://127.0.0.1:8888/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(reponse.status().is_success());
    assert_eq!(Some(0), reponse.content_length());
}

async fn spawn_app() -> std::io::Result<()> {
    zero2prod::run().await
}