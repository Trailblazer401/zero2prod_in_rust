//! tests/health_check.rs

use std::net::TcpListener;
use zero2prod::{configurations::get_configuration, startup::run};
use sqlx::{PgConnection, Connection};

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
    let server = run(listener).expect("Failed to bind addr");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)        
}

#[tokio::test]
async fn subscribe_returns_200_valid() {
    let app_addr = spawn_app();
    let configuration = get_configuration().expect("Failed to read configurations");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let reponse = client
        .post(&format!("{}/subscriptions", &app_addr))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200,reponse.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_bad() {
    let app_addr = spawn_app();
    let client = reqwest::Client::new();
    let test_case = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email")
    ];
    
    for (invalid_body, error_msg) in test_case {
        let reponse = client
            .post(&format!("{}/subscriptions", &app_addr))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            reponse.status().as_u16(),
            "The API did not fail with 400 on requst payload: \"{}\"",
            error_msg
        );
    }
}