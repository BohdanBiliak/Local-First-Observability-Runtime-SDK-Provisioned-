use axum::{response::IntoResponse, routing::get, Router};
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tracing::info;

use crate::metrics::Metrics;

pub async fn start_metrics_server(
    metrics: Arc<Metrics>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/metrics", get(metrics_handler));

    let app = app.with_state(metrics);

    let addr = format!("0.0.0.0:{}", port);
    info!(addr = %addr, "Starting metrics server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn metrics_handler(
    axum::extract::State(metrics): axum::extract::State<Arc<Metrics>>,
) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        [("content-type", "text/plain; version=0.0.4")],
        buffer,
    )
}
