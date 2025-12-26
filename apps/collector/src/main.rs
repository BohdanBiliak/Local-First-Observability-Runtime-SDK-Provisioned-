use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;

use config::Config;

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

    info!(
        rabbitmq_url = %config.rabbitmq_url,
        "Configuration loaded successfully"
    );

    info!("Ready to process telemetry events");
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
