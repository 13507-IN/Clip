use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod api;
mod cache;
mod model;
mod shortener;
mod store;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| ":8080".into());
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".into());

    let store = Arc::new(
        store::Store::new(&database_url).await.expect("failed to open database"),
    );
    let cache = Arc::new(Mutex::new(cache::Cache::new(50000)));
    let counter = Arc::new(AtomicU64::new(0));

    let state = api::AppState {
        store,
        cache,
        base_url: base_url.trim_end_matches('/').to_string(),
        counter,
    };

    let app = Router::new()
        .route("/api/shorten", post(api::shorten))
        .route("/api/stats/{code}", get(api::stats))
        .route("/{code}", get(api::redirect))
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    tracing::info!("server listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
