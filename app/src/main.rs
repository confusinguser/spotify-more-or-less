use crate::api::routes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock};
use tower_http::cors::{Any, CorsLayer};
use crate::storage::Storage;
use crate::config::DataSourceConfig;

mod api;
mod config;
mod extended_history;
mod storage;
mod spotify;
mod types;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load storage based on configuration from environment variables
    // Priority: EXTENDED_HISTORY_PATHS > EXTENDED_HISTORY_PATH > DATA_PATH > default
    let data_source = DataSourceConfig::from_env();
    
    println!("Loading data from: {:?}", data_source);
    
    let storage = data_source.load_storage(10)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load data: {}. Using empty data instead.", e);
            Storage::empty()
        });
    let storage = Arc::new(RwLock::new(storage));

    let socket_address: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let spotify_client = Arc::new(spotify::SpotifyClient::new(
        std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
        std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default(),
    ));
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

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