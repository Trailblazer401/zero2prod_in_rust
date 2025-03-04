//! src/main.rs

use zero2prod::startup;
use zero2prod::configurations::get_configuration;
// use env_logger::Env;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
// use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    
    let application = startup::Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}