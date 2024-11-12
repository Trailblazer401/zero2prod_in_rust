//! main.rs

use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configurations::get_configuration;
use sqlx::{postgres::PgPoolOptions, PgPool};
// use env_logger::Env;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
// use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    // let connection_pool = PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
        // .await
        // .expect("Failed to connect to Postgres");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let addr = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(addr)?;
    run(listener, connection_pool)?.await
}