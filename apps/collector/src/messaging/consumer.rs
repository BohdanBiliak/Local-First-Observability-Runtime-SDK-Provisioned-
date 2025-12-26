use futures::StreamExt;
use lapin::{options::*, types::FieldTable, BasicProperties, Channel};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info, warn};

use super::handler::{HandlerError, MessageHandler};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 5000;
const RETRY_HEADER: &str = "x-retry-count";

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

    pub async fn setup_queues(&self) -> Result<(), ConsumerError> {
        let dlq_name = format!("{}.dlq", self.queue_name);
        let retry_name = format!("{}.retry", self.queue_name);

        let dlq_args = FieldTable::default();
        self.channel
            .queue_declare(
                &dlq_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                dlq_args.clone(),
            )
            .await
            .map_err(|e| ConsumerError::SetupFailed(format!("DLQ setup failed: {}", e)))?;

        let mut retry_args = FieldTable::default();
        retry_args.insert(
            "x-message-ttl".into(),
            lapin::types::AMQPValue::LongInt(RETRY_DELAY_MS as i32),
        );
        retry_args.insert(
            "x-dead-letter-exchange".into(),
            lapin::types::AMQPValue::LongString("".into()),
        );
        retry_args.insert(
            "x-dead-letter-routing-key".into(),
            lapin::types::AMQPValue::LongString(self.queue_name.clone().into()),
        );

        self.channel
            .queue_declare(
                &retry_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                retry_args,
            )
            .await
            .map_err(|e| ConsumerError::SetupFailed(format!("Retry queue setup failed: {}", e)))?;

        let mut main_args = FieldTable::default();
        main_args.insert(
            "x-dead-letter-exchange".into(),
            lapin::types::AMQPValue::LongString("".into()),
        );
        main_args.insert(
            "x-dead-letter-routing-key".into(),
            lapin::types::AMQPValue::LongString(dlq_name.clone().into()),
        );

        self.channel
            .queue_declare(
                &self.queue_name,
                QueueDeclareOptions {
                    durable: true,
                    passive: false,
                    ..Default::default()
                },
                main_args,
            )
            .await
            .map_err(|e| ConsumerError::SetupFailed(format!("Main queue setup failed: {}", e)))?;

        info!(
            queue = %self.queue_name,
            dlq = %dlq_name,
            retry_queue = %retry_name,
            max_retries = MAX_RETRIES,
            retry_delay_ms = RETRY_DELAY_MS,
            "Queue topology configured"
        );

        Ok(())
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
        let routing_key = delivery.routing_key.clone();
        let retry_count = self.get_retry_count(&delivery.properties);
        let data = delivery.data.clone();
        let properties = delivery.properties.clone();

        info!(
            delivery_tag,
            routing_key = routing_key.as_str(),
            retry_count,
            payload_size = data.len(),
            "Processing message"
        );

        match self.handler.handle(delivery).await {
            Ok(()) => {
                info!(delivery_tag, retry_count, "Message processed successfully");

                if let Err(e) = self
                    .channel
                    .basic_ack(delivery_tag, BasicAckOptions::default())
                    .await
                {
                    error!(error = %e, delivery_tag, "Failed to ack message");
                }
            }
            Err(HandlerError::Transient(err)) => {
                if retry_count >= MAX_RETRIES {
                    error!(
                        delivery_tag,
                        retry_count,
                        error = %err,
                        "Max retries exceeded, sending to DLQ"
                    );

                    if let Err(e) = self
                        .channel
                        .basic_reject(delivery_tag, BasicRejectOptions { requeue: false })
                        .await
                    {
                        error!(error = %e, delivery_tag, "Failed to reject to DLQ");
                    }
                } else {
                    warn!(
                        delivery_tag,
                        retry_count,
                        error = %err,
                        "Transient error, scheduling retry"
                    );

                    if let Err(e) = self.retry_message(delivery_tag, data, properties, retry_count).await {
                        error!(error = %e, delivery_tag, "Failed to schedule retry");
                    }
                }
            }
            Err(HandlerError::Permanent(err)) => {
                error!(
                    delivery_tag,
                    error = %err,
                    "Permanent error, rejecting to DLQ"
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

    async fn retry_message(
        &self,
        delivery_tag: u64,
        data: Vec<u8>,
        properties: BasicProperties,
        retry_count: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let retry_queue = format!("{}.retry", self.queue_name);
        let new_retry_count = retry_count + 1;

        let mut headers = properties
            .headers()
            .clone()
            .unwrap_or_else(FieldTable::default);

        headers.insert(
            RETRY_HEADER.into(),
            lapin::types::AMQPValue::LongUInt(new_retry_count),
        );

        let retry_properties = BasicProperties::default()
            .with_headers(headers)
            .with_delivery_mode(2);

        self.channel
            .basic_publish(
                "",
                &retry_queue,
                BasicPublishOptions::default(),
                &data,
                retry_properties,
            )
            .await?
            .await?;

        self.channel
            .basic_ack(delivery_tag, BasicAckOptions::default())
            .await?;

        info!(
            delivery_tag,
            retry_count = new_retry_count,
            retry_queue = %retry_queue,
            "Message scheduled for retry"
        );

        Ok(())
    }

    fn get_retry_count(&self, properties: &BasicProperties) -> u32 {
        properties
            .headers()
            .as_ref()
            .and_then(|headers| headers.inner().get(RETRY_HEADER))
            .and_then(|value| match value {
                lapin::types::AMQPValue::LongUInt(count) => Some(*count),
                _ => None,
            })
            .unwrap_or(0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Failed to start consumer: {0}")]
    ConsumeFailed(String),

    #[error("Failed to setup queue topology: {0}")]
    SetupFailed(String),
}
