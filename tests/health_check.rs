//! tests/health_check.rs

use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{configurations::{get_configuration, DatabaseSettings}, startup::run};
// use sqlx::{PgConnection, Connection};
use sqlx::{Connection, PgConnection, PgPool, Executor};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
// use secrecy::ExposeSecret;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    // let subscriber = get_subscriber("test".into(), "debug".into());
    // init_subscriber(subscriber);
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {    // 此处只要在 test 时指定 TEST_LOG 变量，不论是true还是false由std::env::var返回的结果均是Ok
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

async fn spawn_app() -> TestApp {
    // let subscriber = get_subscriber("test".into(), "debug".into());
    // init_subscriber(subscriber);
    Lazy::force(&TRACING);
    // zero2prod::run().await
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind rand port");
    let port = listener.local_addr().unwrap().port();
    // let server = run(listener).expect("Failed to bind addr");
    // let _ = tokio::spawn(server);
    let addr = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    // let connection_pool = PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres");
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind addr");    // 起了一个 HttpServer，在后续测试中通过调用spawn_app函数来进行httpserver和database connection的建立
    let _ = tokio::spawn(server);

    TestApp {
        address: addr,
        db_pool: connection_pool
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // let mut connection = PgConnection::connect(&config.connection_string_without_db().expose_secret())
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres when migrate database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // spawn_app();
    let client = reqwest::Client::new();
    let test_app: TestApp = spawn_app().await;

    let reponse = client
        .get(format!("{}/health_check", &(test_app.address)))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(reponse.status().is_success());
    assert_eq!(Some(0), reponse.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_valid() {
    // let app_addr = spawn_app();
    let test_app = spawn_app().await;
    // let configuration = get_configuration().expect("Failed to read configurations");
    // let connection_string = configuration.database.connection_string();
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("Failed to connect to Postgres.");
    
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let reponse = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

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
    let client = reqwest::Client::new();
    let test_case = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email")
    ];
    
    for (invalid_body, error_msg) in test_case {
        let reponse = client
            .post(&format!("{}/subscriptions", &test_app.address))
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