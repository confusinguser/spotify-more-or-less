use crate::api::routes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock};
use tower_http::cors::{Any, CorsLayer};

mod api;
mod storage;
mod spotify;
mod types;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get database path from environment variable or use default
    let data_path = std::env::var("data_path").unwrap_or_else(|_| "./tracks.json".to_string());

    let socket_address: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let spotify_client = Arc::new(spotify::SpotifyClient::new(
        std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
        std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default(),
    ));
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    let storage = Arc::new(RwLock::new(storage::Storage::from_file(data_path)?));

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
        .with_state((storage, spotify_client))
        .layer(cors);

    println!("Spotify More Less backend starting on {}", socket_address);
    axum::serve(listener, app.into_make_service()).await
}