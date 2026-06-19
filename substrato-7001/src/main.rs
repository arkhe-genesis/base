//! substrato-7001/src/main.rs
//! Binário principal do Substrato 7001 — x402 Payment Layer + Polar
//!
//! Selo: CATHEDRAL-ARKHE-SUBSTRATO-7001-v2.0.0-2026-06-19

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod metrics_exporter;
mod webhooks;

mod wh {
    pub use crate::webhooks::polar_handler::*;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cathedral_x402_polar=debug".into()),
        )
        .init();

    info!("🏛️ Cathedral ARKHE — Substrato 7001 (x402 + Polar) v2.0.0");

    // 1. Configuração
    let webhook_config = wh::WebhookConfig::from_env()?;
    let dlq: wh::DeadLetterQueue = Arc::new(RwLock::new(Vec::new()));

    // 2. Inicializa handler
    let webhook_handler = Arc::new(
        wh::PolarWebhookHandler::new(webhook_config, Arc::clone(&dlq))
    );

    // 3. Prometheus metrics
    let metrics_port: u16 = std::env::var("POLAR_METRICS_PORT")
        .unwrap_or_else(|_| "9097".to_string())
        .parse()?;
    crate::metrics_exporter::install_metrics_exporter(metrics_port)?;

    // 4. Router Axum
    let app = wh::create_webhook_router(webhook_handler, dlq);

    // 5. Inicia servidor
    let port: u16 = std::env::var("POLAR_WEBHOOK_PORT")
        .unwrap_or_else(|_| "8787".to_string())
        .parse()?;

    info!("🌐 Webhook server em :{}/webhooks/polar", port);
    info!("📊 Metrics em :{}/metrics", metrics_port);
    info!("🔍 DLQ em :{}/webhooks/polar/dlq", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}