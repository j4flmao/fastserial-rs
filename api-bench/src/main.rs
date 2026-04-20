mod handlers;
mod models;

use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("reports").ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,api_bench=debug,tower_http=debug",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting API Benchmark Server on port 8888...");

    let app = Router::new()
        .route("/", get(handlers::index_handler))
        .route("/bench", get(handlers::benchmark_handler))
        .route("/report", get(handlers::report_handler))
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Available endpoints:");
    tracing::info!("  GET /                      - List all saved reports");
    tracing::info!("  GET /bench?sample=N&report=true - Run benchmark and save");
    tracing::info!("  GET /report?sample=N      - View saved report");
    tracing::info!("  GET /health                - Health check");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> impl axum::response::IntoResponse {
    "OK"
}
