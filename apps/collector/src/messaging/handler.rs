use async_trait::async_trait;
use lapin::message::Delivery;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, delivery: Delivery) -> Result<(), HandlerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Transient error (will retry): {0}")]
    Transient(String),

    #[error("Permanent error (will not retry): {0}")]
    Permanent(String),
}
