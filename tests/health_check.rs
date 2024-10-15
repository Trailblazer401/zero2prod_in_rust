//! tests/health_check.rs

use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // spawn_app();
    let client = reqwest::Client::new();

    let reponse = client
        .get(format!("{}/health_check", &spawn_app()))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(reponse.status().is_success());
    assert_eq!(Some(0), reponse.content_length());
}

fn spawn_app() -> String {
    // zero2prod::run().await
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind addr");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind addr");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}