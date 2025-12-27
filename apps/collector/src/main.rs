use std::sync::Arc;
use async_trait::async_trait;
use lapin::message::Delivery;
use tokio::sync::Notify;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod config;

use config::Config;
use observability_collector::messaging::{
    ChannelProvider, Consumer, HandlerError, MessageHandler, RabbitMqConnection,
};
use observability_collector::metrics::{server::start_metrics_server, Metrics};

struct TelemetryHandler;

const EVENT_VERSION_HEADER: &str = "x-event-version";

#[async_trait]
impl MessageHandler for TelemetryHandler {
    async fn handle(&self, delivery: Delivery) -> Result<(), HandlerError> {
        let payload = String::from_utf8_lossy(&delivery.data);
        
        // Extract version from headers
        let version = delivery
            .properties
            .headers()
            .as_ref()
            .and_then(|headers| headers.inner().get(EVENT_VERSION_HEADER))
            .and_then(|value| match value {
                lapin::types::AMQPValue::LongString(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "v1".to_string());

        info!(
            routing_key = delivery.routing_key.as_str(),
            version = %version,
            payload_preview = %payload.chars().take(100).collect::<String>(),
            "Handling telemetry message"
        );

        // Version-based routing
        match version.as_str() {
            "v1" => self.handle_v1(&payload),
            _ => {
                return Err(HandlerError::Permanent(format!(
                    "Unsupported event version: {}. Only v1 is supported.",
                    version
                )));
            }
        }
    }
}

impl TelemetryHandler {
    fn handle_v1(&self, payload: &str) -> Result<(), HandlerError> {
        // Test error simulation
        if payload.contains("\"fail\":\"transient\"") {
            return Err(HandlerError::Transient("Simulated transient failure".to_string()));
        }

        if payload.contains("\"fail\":\"permanent\"") {
            return Err(HandlerError::Permanent("Simulated permanent failure".to_string()));
        }

        // Parse and validate v1 schema
        match serde_json::from_str::<serde_json::Value>(payload) {
            Ok(json) => {
                // Basic v1 validation
                if !json.get("eventType").is_some() {
                    return Err(HandlerError::Permanent(
                        "Missing required field: eventType".to_string(),
                    ));
                }
                if !json.get("payload").is_some() {
                    return Err(HandlerError::Permanent(
                        "Missing required field: payload".to_string(),
                    ));
                }
                
                info!("Successfully processed v1 event");
                Ok(())
            }
            Err(e) => {
                Err(HandlerError::Permanent(format!(
                    "Invalid JSON payload: {}",
                    e
                )))
            }
        }
    }
}

#[tokio::main]
async fn main() {
    setup_panic_handler();
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    setup_logging(&config.rust_log);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        service_name = %config.service_name,
        "Observability Collector starting"
    );

    let rabbitmq = match RabbitMqConnection::connect(config.rabbitmq_url.clone()).await {
        Ok(conn) => {
            info!("RabbitMQ connection established");
            conn
        }
        Err(e) => {
            eprintln!("Failed to connect to RabbitMQ: {}", e);
            std::process::exit(1);
        }
    };

    let channel = match ChannelProvider::create_channel(rabbitmq.get_connection()).await {
        Ok(ch) => {
            info!("RabbitMQ channel created and configured");
            ch
        }
        Err(e) => {
            eprintln!("Failed to create RabbitMQ channel: {}", e);
            std::process::exit(1);
        }
    };

    let shutdown = Arc::new(Notify::new());
    let shutdown_clone = shutdown.clone();

    let metrics = Metrics::new().expect("Failed to create metrics");

    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = start_metrics_server(metrics_clone, 9090).await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    let handler = Arc::new(TelemetryHandler);
    let consumer = Consumer::new(
        channel,
        "telemetry".to_string(),
        format!("{}-consumer", config.service_name),
        handler,
        shutdown_clone,
        metrics.clone(),
    );

    if let Err(e) = consumer.setup_queues().await {
        eprintln!("Failed to setup queue topology: {}", e);
        std::process::exit(1);
    }

    let consumer_handle = tokio::spawn(async move {
        if let Err(e) = consumer.start().await {
            eprintln!("Consumer error: {}", e);
        }
    });

    info!("Ready to process telemetry events");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    warn!("Shutdown signal received, cleaning up...");

    shutdown.notify_one();

    if let Err(e) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        consumer_handle,
    )
    .await
    {
        warn!(error = ?e, "Consumer shutdown timeout");
    }

    if let Err(e) = rabbitmq.shutdown().await {
        eprintln!("Error during shutdown: {}", e);
    }

    info!("Observability Collector stopped");
}

fn setup_logging(rust_log: &str) {
    let log_level = match rust_log.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}

fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        eprintln!("PANIC: {} at {}", message, location);
        eprintln!("Thread: {:?}", std::thread::current().name());
        eprintln!("Backtrace:");
        
    }));
}
