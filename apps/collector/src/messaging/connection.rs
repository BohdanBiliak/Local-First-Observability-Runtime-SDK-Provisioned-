use lapin::{Connection, ConnectionProperties};
use tracing::{error, info};

pub struct RabbitMqConnection {
    connection: Connection,
    url: String,
}

impl RabbitMqConnection {
    pub async fn connect(url: String) -> Result<Self, ConnectionError> {
        info!(url = %url, "Connecting to RabbitMQ");

        let connection = Connection::connect(&url, ConnectionProperties::default())
            .await
            .map_err(|e| {
                error!(error = %e, url = %url, "Failed to connect to RabbitMQ");
                ConnectionError::ConnectionFailed(e.to_string())
            })?;

        info!(url = %url, "Successfully connected to RabbitMQ");

        Ok(Self { connection, url })
    }

    pub fn get_connection(&self) -> &Connection {
        &self.connection
    }

    pub fn is_connected(&self) -> bool {
        self.connection.status().connected()
    }

    pub async fn shutdown(self) -> Result<(), ConnectionError> {
        info!(url = %self.url, "Shutting down RabbitMQ connection");

        self.connection
            .close(200, "Normal shutdown")
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to close RabbitMQ connection gracefully");
                ConnectionError::ShutdownFailed(e.to_string())
            })?;

        info!("RabbitMQ connection closed successfully");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to connect to RabbitMQ: {0}")]
    ConnectionFailed(String),

    #[error("Failed to shutdown connection gracefully: {0}")]
    ShutdownFailed(String),
}
