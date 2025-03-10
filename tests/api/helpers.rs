//! tests/api/helpers.rs

use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configurations::{get_configuration, DatabaseSettings};
// use sqlx::{PgConnection, Connection};
use sqlx::{Connection, PgConnection, PgPool, Executor};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use zero2prod::startup;
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
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        // reqwest::Client::new()
        self.api_client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks {
            html,
            plain_text,
        }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        // reqwest::Client::new()
        self.api_client
            .post(&format!("{}/newsletters", &self.address))
            // .basic_auth(Uuid::new_v4().to_string(), Some(Uuid::new_v4().to_string()))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    // pub async fn test_user(&self) -> (String, String) {
    //     let row = sqlx::query!("SELECT username, password FROM users LIMIT 1")
    //         .fetch_one(&self.db_pool)
    //         .await
    //         .expect("Failed to fetch test user");
    //     (row.username, row.password)
    // }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        // reqwest::Client::builder()
        self.api_client
            // .redirect(reqwest::redirect::Policy::none())
            // .build()
            // .unwrap()
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_login_html(&self) -> String {
        // reqwest::Client::new()
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
            .text()
            .await
            .unwrap()
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
 
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard()
            .await
            .text()
            .await
            .unwrap()
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/password", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response 
        where Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/admin/password", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password().await.text().await.unwrap()
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn create() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
            // password: "everythinghastostartsomewhere".into(),
        }
    }

    pub async fn save(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let passwd_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2,1, None).unwrap(),
        )
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        // dbg!(&passwd_hash);
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            passwd_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to save test user");
        
    }    
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    // zero2prod::run().await

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c = get_configuration().expect("Failed to get configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = startup::Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.port());
    let port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app =TestApp {
        address,
        db_pool: startup::get_connection_pool(&configuration.database),
        email_server,
        port,
        test_user: TestUser::create(),
        api_client: client,
    };
    test_app.test_user.save(&test_app.db_pool).await;
    
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
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

// async fn add_test_user(db_pool: &PgPool) {
//     sqlx::query!(
//         "INSERT INTO users (user_id, username, password) VALUES ($1, $2, $3)",
//         Uuid::new_v4(),
//         Uuid::new_v4().to_string(),
//         Uuid::new_v4().to_string(),
//     )
//     .execute(db_pool)
//     .await
//     .expect("Failed to create test user");
// }

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}