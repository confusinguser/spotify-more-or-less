use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use axum_macros::debug_handler;
use tokio::sync::RwLock;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::spotify::SpotifyClient;
use crate::types::{TrackInfo, TwoTracksResponse, AlbumImageResponse};

pub(crate) fn router() -> axum::Router<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)> {
    let (app_router, openapi) = OpenApiRouter::new()
        .routes(routes!(get_random_track))
        .routes(routes!(get_two_random_tracks))
        .routes(routes!(get_album_image))
        .split_for_parts();
    app_router.route(
        "/openapi.json",
        axum::routing::get(move || async { Json(openapi) }),
    )
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/random",
    responses(
        (status = 200, description = "", body = TrackInfo),
    )
)]
pub async fn get_random_track(
    State((storage, spotify_client)): State<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)>,
) -> Result<Json<TrackInfo>, StatusCode> {
    storage.write().await
        .random_track(&spotify_client).await
        .map(|track| Json(track.clone()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/random/two",
    responses(
        (status = 200, description = "Two random tracks", body = TwoTracksResponse),
    )
)]
pub async fn get_two_random_tracks(
    State((storage, spotify_client)): State<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)>,
) -> Result<Json<TwoTracksResponse>, (StatusCode, String)> {
    let track1 = storage.write().await
        .random_track(&spotify_client).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let track2 = storage.write().await
        .random_track(&spotify_client).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TwoTracksResponse {
        track1,
        track2,
    }))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/{track_id}/album-image",
    params(
        ("track_id" = String, Path, description = "Spotify track ID")
    ),
    responses(
        (status = 200, description = "Album image URL", body = AlbumImageResponse),
    )
)]
pub async fn get_album_image(
    Path(track_id): Path<String>,
    State((_, spotify_client)): State<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)>,
) -> Result<Json<AlbumImageResponse>, StatusCode> {
    let track = spotify_client
        .get_track(&track_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let image_url = track.album.images.first().map(|img| img.url.clone());

    Ok(Json(AlbumImageResponse {
        track_id,
        album_image_url: image_url,
    }))
}

