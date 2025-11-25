use crate::api::routes;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use crate::storage::TracksJson;

mod api;
mod storage;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get database path from environment variable or use default
    let data_path = std::env::var("data_path").unwrap_or_else(|_| "./tracks.json".to_string());

    let socket_address: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    let f = std::fs::File::open(data_path)?;
    let tracks: TracksJson = serde_json::from_reader(f)?;
    let storage = Arc::new(storage::Storage { tracks });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
            axum::http::Method::PUT,
        ])
        .allow_headers(Any);

    let app = axum::Router::new()
        .merge(routes::router())
        .with_state(storage)
        .layer(cors);

    println!("Spotify More Less backend starting on {}", socket_address);
    axum::serve(listener, app.into_make_service()).await
}