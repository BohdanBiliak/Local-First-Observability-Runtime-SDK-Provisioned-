use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod config;

use config::Config;
use observability_collector::messaging::{ChannelProvider, RabbitMqConnection};

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

    info!("Ready to process telemetry events");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    warn!("Shutdown signal received, cleaning up...");
    if let Err(e) = ChannelProvider::close_channel(channel).await {
        eprintln!("Error closing channel: {}", e);
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
