use lapin::{Channel, Connection};
use tracing::{error, info};

pub struct ChannelProvider;

impl ChannelProvider {

    pub async fn create_channel(connection: &Connection) -> Result<Channel, ChannelError> {
        info!("Creating RabbitMQ channel");

        let channel = connection
            .create_channel()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to create RabbitMQ channel");
                ChannelError::CreationFailed(e.to_string())
            })?;

        info!(channel_id = channel.id(), "Channel created successfully");

        info!(prefetch_count = 10, "Configuring channel QoS");
        
        channel
            .basic_qos(10, Default::default())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to configure channel QoS");
                ChannelError::QoSConfigurationFailed(e.to_string())
            })?;

        info!(
            channel_id = channel.id(),
            prefetch_count = 10,
            "Channel QoS configured successfully"
        );

        Ok(channel)
    }

    pub async fn close_channel(channel: Channel) -> Result<(), ChannelError> {
        let channel_id = channel.id();
        info!(channel_id, "Closing RabbitMQ channel");

        channel
            .close(200, "Normal shutdown")
            .await
            .map_err(|e| {
                error!(error = %e, channel_id, "Failed to close channel gracefully");
                ChannelError::CloseFailed(e.to_string())
            })?;

        info!(channel_id, "Channel closed successfully");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Failed to create channel: {0}")]
    CreationFailed(String),

    #[error("Failed to configure channel QoS: {0}")]
    QoSConfigurationFailed(String),

    #[error("Failed to close channel: {0}")]
    CloseFailed(String),
}
