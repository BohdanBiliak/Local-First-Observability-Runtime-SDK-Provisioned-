use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info, warn};

use super::handler::{HandlerError, MessageHandler};

pub struct Consumer {
    channel: Channel,
    queue_name: String,
    consumer_tag: String,
    handler: Arc<dyn MessageHandler>,
    shutdown: Arc<Notify>,
}

impl Consumer {
    pub fn new(
        channel: Channel,
        queue_name: String,
        consumer_tag: String,
        handler: Arc<dyn MessageHandler>,
        shutdown: Arc<Notify>,
    ) -> Self {
        Self {
            channel,
            queue_name,
            consumer_tag,
            handler,
            shutdown,
        }
    }

    pub async fn start(self) -> Result<(), ConsumerError> {
        info!(
            queue = %self.queue_name,
            consumer_tag = %self.consumer_tag,
            "Starting RabbitMQ consumer"
        );

        let mut consumer = self
            .channel
            .basic_consume(
                &self.queue_name,
                &self.consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!(error = %e, queue = %self.queue_name, "Failed to start consumer");
                ConsumerError::ConsumeFailed(e.to_string())
            })?;

        info!(
            queue = %self.queue_name,
            consumer_tag = %self.consumer_tag,
            "Consumer started successfully"
        );

        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    info!(
                        consumer_tag = %self.consumer_tag,
                        "Shutdown signal received, stopping consumer"
                    );
                    break;
                }

                delivery = consumer.next() => {
                    match delivery {
                        Some(Ok(delivery)) => {
                            self.process_message(delivery).await;
                        }
                        Some(Err(e)) => {
                            error!(error = %e, "Error receiving message from RabbitMQ");
                        }
                        None => {
                            warn!("Consumer stream ended");
                            break;
                        }
                    }
                }
            }
        }

        info!(consumer_tag = %self.consumer_tag, "Consumer stopped");
        Ok(())
    }

    async fn process_message(&self, delivery: lapin::message::Delivery) {
        let delivery_tag = delivery.delivery_tag;
        let routing_key = delivery.routing_key.as_str();

        info!(
            delivery_tag,
            routing_key,
            payload_size = delivery.data.len(),
            "Processing message"
        );

        match self.handler.handle(delivery).await {
            Ok(()) => {
                info!(delivery_tag, "Message processed successfully");

                if let Err(e) = self
                    .channel
                    .basic_ack(delivery_tag, BasicAckOptions::default())
                    .await
                {
                    error!(error = %e, delivery_tag, "Failed to ack message");
                }
            }
            Err(HandlerError::Transient(err)) => {
                warn!(
                    delivery_tag,
                    error = %err,
                    "Transient error, nacking with requeue"
                );

                if let Err(e) = self
                    .channel
                    .basic_nack(delivery_tag, BasicNackOptions {
                        requeue: true,
                        ..Default::default()
                    })
                    .await
                {
                    error!(error = %e, delivery_tag, "Failed to nack message");
                }
            }
            Err(HandlerError::Permanent(err)) => {
                error!(
                    delivery_tag,
                    error = %err,
                    "Permanent error, rejecting without requeue"
                );

                if let Err(e) = self
                    .channel
                    .basic_reject(delivery_tag, BasicRejectOptions { requeue: false })
                    .await
                {
                    error!(error = %e, delivery_tag, "Failed to reject message");
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Failed to start consumer: {0}")]
    ConsumeFailed(String),
}
