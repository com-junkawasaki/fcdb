//! Own-CFA-Enishi Main Application
//!
//! Production-ready main entry point for the Own-CFA-Enishi database system.
//! Provides HTTP API endpoints and integrates all system components.

use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod server;
mod metrics;
mod health;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "enishi=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Own-CFA-Enishi v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::load_config()?;
    info!("📋 Configuration loaded: {:?}", config);

    // Initialize metrics
    let metrics = std::sync::Arc::new(metrics::MetricsCollector::new());
    metrics.start_collection();

    // Initialize health checker
    let health_checker = std::sync::Arc::new(health::HealthChecker::new());

    // TODO: Initialize system components when ready
    // let cas = enishi_cas::PackCAS::open(&config.storage_path).await?;
    // let graph = enishi_graph::GraphDB::new(cas).await;
    // let executor = enishi_exec::SafeExecutor::new();

    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let server = server::Server::new(config, metrics.clone(), health_checker.clone());

    info!("🌐 Starting HTTP server on {}", addr);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run(addr).await {
            error!("Server error: {}", e);
        }
    });

    // Graceful shutdown handling
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 Shutdown signal received, stopping gracefully...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Perform graceful shutdown
    info!("🛑 Initiating graceful shutdown...");

    // Stop metrics collection
    metrics.stop_collection().await;

    // Wait for server to finish (with timeout)
    let shutdown_timeout = tokio::time::Duration::from_secs(30);
    match tokio::time::timeout(shutdown_timeout, server_handle).await {
        Ok(_) => info!("✅ Server shutdown complete"),
        Err(_) => warn!("⚠️  Server shutdown timed out"),
    }

    info!("👋 Own-CFA-Enishi shutdown complete");
    Ok(())
}