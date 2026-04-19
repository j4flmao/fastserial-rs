mod auth;
mod db;
mod error;
mod fastjson;
mod handlers;
mod models;
mod routes;

use axum::Router;
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,sample_axum=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting server...");

    tracing::info!(
        "Connecting to database at {}...",
        env::var("DATABASE_URL").unwrap_or_default()
    );
    let pool = db::init_db().await;
    tracing::info!("Database connected!");

    // Initialize database schema and seed data
    tracing::info!("Initializing database schema...");
    db::schema::init_schema(&pool).await?;

    tracing::info!("Seeding initial data...");
    db::seeds::seed_data(&pool).await?;

    tracing::info!("Database ready!");

    let app = Router::new()
        .nest("/api/users", routes::user::create_user_routes(pool.clone()))
        .nest("/api/posts", routes::post::create_post_routes(pool.clone()))
        .nest(
            "/api/categories",
            routes::category::create_category_routes(pool.clone()),
        )
        .layer(CorsLayer::permissive());

    let port = env::var("PORT").unwrap_or_else(|_| "8082".to_string());
    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>()?;

    tracing::info!("Server listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
