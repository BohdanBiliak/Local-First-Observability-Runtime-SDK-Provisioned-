use prometheus::{
    Counter, CounterVec, Gauge, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::Arc;

pub mod server;

pub struct Metrics {
    pub messages_processed_total: CounterVec,
    pub messages_failed_total: CounterVec,
    pub messages_retried_total: Counter,
    pub messages_dlq_total: Counter,
    pub message_processing_duration_seconds: HistogramVec,
    pub active_consumers: Gauge,
    pub registry: Registry,
}

impl Metrics {
    pub fn new() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let registry = Registry::new();

        let messages_processed_total = CounterVec::new(
            Opts::new(
                "collector_messages_processed_total",
                "Total number of messages successfully processed",
            ),
            &["queue", "routing_key"],
        )?;

        let messages_failed_total = CounterVec::new(
            Opts::new(
                "collector_messages_failed_total",
                "Total number of messages that failed processing",
            ),
            &["queue", "error_type"],
        )?;

        let messages_retried_total = Counter::new(
            "collector_messages_retried_total",
            "Total number of messages sent to retry queue",
        )?;

        let messages_dlq_total = Counter::new(
            "collector_messages_dlq_total",
            "Total number of messages sent to dead letter queue",
        )?;

        let message_processing_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "collector_message_processing_duration_seconds",
                "Time taken to process a message",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["queue", "status"],
        )?;

        let active_consumers = Gauge::new(
            "collector_active_consumers",
            "Number of active consumer loops",
        )?;

        registry.register(Box::new(messages_processed_total.clone()))?;
        registry.register(Box::new(messages_failed_total.clone()))?;
        registry.register(Box::new(messages_retried_total.clone()))?;
        registry.register(Box::new(messages_dlq_total.clone()))?;
        registry.register(Box::new(message_processing_duration_seconds.clone()))?;
        registry.register(Box::new(active_consumers.clone()))?;

        Ok(Arc::new(Self {
            messages_processed_total,
            messages_failed_total,
            messages_retried_total,
            messages_dlq_total,
            message_processing_duration_seconds,
            active_consumers,
            registry,
        }))
    }
}
