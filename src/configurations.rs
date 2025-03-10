//! src/configurations.rs
 
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, Clone)]
// 使用派生 trait 宏自动为结构体实现指定 trait，相当于使用 impl Clone for struct 并递归地为结构体中的每一类型进行 .clone() 方法调用（前提是每一类型都实现了Clone trait）
pub struct Settings {
    pub database: DatabaseSettings,
    // pub application_port: u16
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
    pub redis_uri: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_millisec: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub ssl_required: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        // "local".into(): convert [ &'static str ](slice) to [ String ]
        .try_into()
        // try_into(): convert [ String ](from unwrap_or_else return) to [ Ok(Environment) ] or [ Err(String) ] 
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        // .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .add_source(config::File::from(configuration_directory.join("base.yaml")))
        .add_source(config::File::from(configuration_directory.join(&environment_filename)))
        .add_source(config::Environment::with_prefix("APP").prefix_separator("_").separator("__"))
        // use env variable like "APP_APPLICATION__PORT" to overwrite the Setting values in the configuration YAML files
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Prodution,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Prodution => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;    // literally claim the return error type

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Prodution),
            other => Err(format!("{} is not a supported environment. Use either 'local' or 'production'.", other)),
        }
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.clone().log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.ssl_required {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_millisec)
    }
}