use crate::api::routes;
use crate::config::DataSourceConfig;
use crate::storage::{Storage, UserStorages};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

mod api;
mod config;
mod extended_history;
mod spotify;
mod storage;
mod types;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let data_source = DataSourceConfig::from_env();

    println!("Loading data from: {:?}", data_source);

    let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() {
        eprintln!("Warning: SPOTIFY_CLIENT_ID is not set. Spotify API requests may fail.");
    }
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default();
    if client_secret.is_empty() {
        eprintln!("Warning: SPOTIFY_CLIENT_SECRET is not set. Spotify API requests may fail.");
    }
    let spotify_client = Arc::new(spotify::SpotifyClient::new(client_id, client_secret));

    // Try multi-user mode first (DATA_DIR with subdirectories)
    let user_storages: UserStorages = if let Ok(users) = data_source.load_multi_user_storages(10) {
        if !users.is_empty() {
            println!("Multi-user mode: loaded {} user(s)", users.len());
            let map: HashMap<String, Arc<RwLock<Storage>>> = users
                .into_iter()
                .map(|(k, v)| (k, Arc::new(RwLock::new(v))))
                .collect();
            Arc::new(RwLock::new(map))
        } else {
            // Fall back to single-user mode with a default user name
            println!("No user subdirectories found; loading as single-user");
            let storage = data_source.load_storage(10).unwrap_or_else(|e| {
                eprintln!("Failed to load data: {}. Using empty data instead.", e);
                Storage::empty()
            });
            let mut map = HashMap::new();
            map.insert("default".to_string(), Arc::new(RwLock::new(storage)));
            Arc::new(RwLock::new(map))
        }
    } else {
        // load_multi_user_storages failed (e.g. path is a file) – single-user fallback
        let storage = data_source.load_storage(10).unwrap_or_else(|e| {
            eprintln!("Failed to load data: {}. Using empty data instead.", e);
            Storage::empty()
        });
        let mut map = HashMap::new();
        map.insert("default".to_string(), Arc::new(RwLock::new(storage)));
        Arc::new(RwLock::new(map))
    };

    let socket_address: SocketAddr = "0.0.0.0:8000".parse().unwrap();
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
        .with_state((user_storages, spotify_client))
        .layer(cors);

    println!("Spotify More Less backend starting on {}", socket_address);
    axum::serve(listener, app.into_make_service()).await
}
