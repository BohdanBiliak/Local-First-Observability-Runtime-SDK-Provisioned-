use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub rabbitmq_url: String,
    pub service_name: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let rabbitmq_url = env::var("RABBITMQ_URL")
            .map_err(|_| ConfigError::MissingRequired("RABBITMQ_URL"))?;

        let service_name = env::var("SERVICE_NAME")
            .map_err(|_| ConfigError::MissingRequired("SERVICE_NAME"))?;

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            rabbitmq_url,
            service_name,
            rust_log,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingRequired(&'static str),
}
